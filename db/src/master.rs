/* Copyright (c) 2018 University of Utah
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR(S) DISCLAIM ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL AUTHORS BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

use std::collections::HashMap;
use std::mem::transmute;
use std::rc::Rc;
use std::sync::Arc;

use super::alloc::Allocator;
use super::common::{TableId, TenantId, PACKET_UDP_LEN};
use super::container::Container;
use super::context::Context;
use super::ext::*;
use super::native::Native;
use super::service::Service;
use super::task::{Task, TaskPriority};
use super::tenant::Tenant;
use super::wireformat::*;

use e2d2::common::EmptyMetadata;
use e2d2::headers::UdpHeader;
use e2d2::interface::Packet;

use spin::RwLock;

// The number of buckets in the `tenants` hashtable inside of Master.
const TENANT_BUCKETS: usize = 32;

/// The primary service in Sandstorm. Master is responsible managing tenants, extensions, and
/// the database. It implements the Service trait, allowing it to generate schedulable tasks
/// for data and extension related RPC requests.
pub struct Master {
    // A Map of all tenants in the system. Since Sandstorm is a multi-tenant system, most RPCs
    // will require a lookup on this map.
    tenants: [RwLock<HashMap<TenantId, Arc<Tenant>>>; TENANT_BUCKETS],

    // An extension manager maintaining state concerning extensions loaded into the system.
    // Required to retrieve and determine if an extension belongs to a particular tenant while
    // handling an invocation request.
    extensions: ExtensionManager,

    // Manager of the table heap. Required to allow writes to the database.
    heap: Arc<Allocator>,
}

// Implementation of methods on Master.
impl Master {
    /// Creates and returns a new Master service.
    ///
    /// # Return
    ///
    /// A Master service capable of creating schedulable tasks out of RPC requests.
    pub fn new() -> Master {
        Master {
            // Cannot use copy constructor because of the Arc<Tenant>.
            tenants: [
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
                RwLock::new(HashMap::new()),
            ],
            extensions: ExtensionManager::new(),
            heap: Arc::new(Allocator::new()),
        }
    }

    /// Adds a tenant and a table full of objects.
    ///
    /// # Arguments
    ///
    /// * `tenant_id`: Identifier of the tenant to be added. Any existing tenant with the same
    ///                identifier will be overwritten.
    /// * `table_id`:  Identifier of the table to be added to the tenant. This table will contain
    ///                all the objects.
    /// * `num`:       The number of objects to be added to the data table.
    pub fn fill_test(&self, tenant_id: TenantId, table_id: TableId, num: u32) {
        // Create a tenant containing the table.
        let tenant = Tenant::new(tenant_id);
        tenant.create_table(table_id);

        let table = tenant
            .get_table(table_id)
            .expect("Failed to init test table.");

        let mut key = vec![0; 30];
        let mut val = vec![0; 100];

        // Allocate objects, and fill up the above table. Each object consists of a 30 Byte key
        // and a 100 Byte value.
        for i in 1..(num + 1) {
            let temp: [u8; 4] = unsafe { transmute(i.to_le()) };
            &key[0..4].copy_from_slice(&temp);
            &val[0..4].copy_from_slice(&temp);

            let obj = self.heap
                .object(tenant_id, table_id, &key, &val)
                .expect("Failed to create test object.");
            table.put(obj.0, obj.1);
        }

        // Add the tenant.
        self.insert_tenant(tenant);
    }

    /// Loads the get(), put(), and tao() extensions.
    ///
    /// # Arguments
    ///
    /// * `tenant`: Identifier of the tenant to load the extension for.
    pub fn load_test(&self, tenant: TenantId) {
        // Load the get() extension.
        let name = "../ext/get/target/release/libget.so";
        if self.extensions.load(name, tenant, "get") == false {
            panic!("Failed to load get() extension.");
        }

        // Load the put() extension.
        let name = "../ext/put/target/release/libput.so";
        if self.extensions.load(name, tenant, "put") == false {
            panic!("Failed to load put() extension.");
        }

        // Load the tao() extension.
        let name = "../ext/tao/target/release/libtao.so";
        if self.extensions.load(name, tenant, "tao") == false {
            panic!("Failed to load tao() extension.");
        }
    }

    /// Loads the get(), put(), and tao() extensions once, and shares them across multiple tenants.
    ///
    /// # Arguments
    ///
    /// * `tenants`: The number of tenants that should share the above three extensions.
    pub fn load_test_shared(&self, tenants: u32) {
        // First, load up the get, put, and tao extensions for tenant 1.
        self.load_test(0);

        // Next, share these extensions with the other tenants.
        for tenant in 1..tenants {
            // Share the get() extension.
            if self.extensions.share(0, tenant, "get") == false {
                panic!("Failed to share get() extension.");
            }

            // Share the put() extension.
            if self.extensions.share(0, tenant, "put") == false {
                panic!("Failed to share put() extension.");
            }

            // Share the tao() extension.
            if self.extensions.share(0, tenant, "tao") == false {
                panic!("Failed to share tao() extension.");
            }
        }
    }

    /// This method returns a handle to a tenant if it exists.
    ///
    /// # Arguments
    ///
    /// * `tenant_id`: The identifier for the tenant to be returned.
    ///
    /// # Return
    ///
    /// An atomic reference counted handle to the tenant if it exists.
    fn get_tenant(&self, tenant_id: TenantId) -> Option<Arc<Tenant>> {
        // Acquire a read lock. The bucket is determined by the least significant byte of the
        // tenant id.
        let bucket = (tenant_id & 0xff) as usize & (TENANT_BUCKETS - 1);
        let map = self.tenants[bucket].read();

        // Lookup, and return the tenant if it exists.
        map.get(&tenant_id)
            .and_then(|tenant| Some(Arc::clone(tenant)))
    }

    /// This method adds a tenant to Master.
    ///
    /// # Arguments
    ///
    /// * `tenant`: The tenant to be added.
    fn insert_tenant(&self, tenant: Tenant) {
        // Acquire a write lock. The bucket is determined by the least significant byte of the
        // tenant id.
        let bucket = (tenant.id() & 0xff) as usize & (TENANT_BUCKETS - 1);
        let mut map = self.tenants[bucket].write();

        // Insert the tenant and return.
        map.insert(tenant.id(), Arc::new(tenant));
    }

    /// Handles the Get() RPC request.
    ///
    /// A hash table lookup is performed on a supplied tenant id, table id, and key. If successfull,
    /// the result of the lookup is written into a response packet, and the response header is
    /// updated. In the case of a failure, the response header is updated with a status indicating
    /// the reason for the failure.
    ///
    /// # Arguments
    ///
    /// * `req`: The RPC request packet sent by the client, parsed upto it's UDP header.
    /// * `res`: The RPC response packet, with pre-allocated headers upto UDP.
    ///
    /// # Return
    ///
    /// A Native task that can be scheduled by the database. In the case of an error, the passed
    /// in request and response packets are returned with the response status appropriately set.
    #[allow(unreachable_code)]
    #[allow(unused_assignments)]
    fn get(
        &self,
        req: Packet<UdpHeader, EmptyMetadata>,
        res: Packet<UdpHeader, EmptyMetadata>,
    ) -> Result<
        Box<Task>,
        (
            Packet<UdpHeader, EmptyMetadata>,
            Packet<UdpHeader, EmptyMetadata>,
        ),
    > {
        // First, parse the request packet.
        let req = req.parse_header::<GetRequest>();

        // Read fields off the request header.
        let mut tenant_id: TenantId = 0;
        let mut table_id: TableId = 0;
        let mut key_length = 0;
        let mut rpc_stamp = 0;

        {
            let hdr = req.get_header();
            tenant_id = hdr.common_header.tenant as TenantId;
            table_id = hdr.table_id as TableId;
            key_length = hdr.key_length;
            rpc_stamp = hdr.common_header.stamp
        }

        // Next, add a header to the response packet.
        let mut res = res.push_header(&GetResponse::new(rpc_stamp))
            .expect("Failed to setup GetResponse");

        // If the payload size is less than the key length, return an error.
        if req.get_payload().len() < key_length as usize {
            res.get_mut_header().common_header.status = RpcStatus::StatusMalformedRequest;
            return Err((
                req.deparse_header(PACKET_UDP_LEN as usize),
                res.deparse_header(PACKET_UDP_LEN as usize),
            ));
        }

        // Lookup the tenant, and get a handle to the allocator. Required to avoid capturing a
        // reference to Master in the generator below.
        let tenant = self.get_tenant(tenant_id);
        let alloc = self.heap.clone();

        // Create a generator for this request.
        let gen = Box::new(move || {
            let mut status: RpcStatus = RpcStatus::StatusTenantDoesNotExist;

            let outcome =
                // Check if the tenant exists. If it does, then check if the
                // table exists, and update the status of the rpc.
                tenant.and_then(| tenant | {
                                status = RpcStatus::StatusTableDoesNotExist;
                                tenant.get_table(table_id)
                            })
                // If the table exists, lookup the provided key, and update
                // the status of the rpc.
                .and_then(| table | {
                                status = RpcStatus::StatusObjectDoesNotExist;
                                let (key, _) = req.get_payload().split_at(key_length as usize);
                                table.get(key)
                            })
                // If the lookup succeeded, obtain the value, and update the
                // status of the rpc.
                .and_then(| object | {
                                status = RpcStatus::StatusInternalError;
                                alloc.resolve(object)
                            })
                // If the value was obtained, then write to the response packet
                // and update the status of the rpc.
                .and_then(| (_k, value) | {
                                status = RpcStatus::StatusInternalError;
                                res.add_to_payload_tail(value.len(), &value[..]).ok()
                            })
                // If the value was written to the response payload,
                // update the status of the rpc.
                .and_then(| _ | {
                                status = RpcStatus::StatusOk;
                                Some(())
                            });

            match outcome {
                // The RPC completed successfully. Update the response header with
                // the status and value length.
                Some(()) => {
                    let val_len = res.get_payload().len() as u32;

                    let hdr: &mut GetResponse = res.get_mut_header();
                    hdr.value_length = val_len;
                    hdr.common_header.status = status;
                }

                // The RPC failed. Update the response header with the status.
                None => {
                    res.get_mut_header().common_header.status = status;
                }
            }

            // Deparse request and response packets down to UDP, and return from the generator.
            return Some((
                req.deparse_header(PACKET_UDP_LEN as usize),
                res.deparse_header(PACKET_UDP_LEN as usize),
            ));

            // XXX: This yield is required to get the compiler to compile this closure into a
            // generator. It is unreachable and benign.
            yield 0;
        });

        // Return a native task.
        return Ok(Box::new(Native::new(TaskPriority::REQUEST, gen)));
    }

    /// Handles the put() RPC request.
    ///
    /// If the issuing tenant is valid, a new key-value pair is allocated, and inserted into a
    /// table if it exists.
    ///
    /// # Arguments
    ///
    /// * `req`: The RPC request packet sent by the client, parsed upto it's UDP header.
    /// * `res`: The RPC response packet, with pre-allocated headers upto UDP.
    ///
    /// # Return
    ///
    /// A Native task that can be scheduled by the database. In the case of an error, the passed
    /// in request and response packets are returned with the response status appropriately set.
    #[allow(unreachable_code)]
    #[allow(unused_assignments)]
    fn put(
        &self,
        req: Packet<UdpHeader, EmptyMetadata>,
        res: Packet<UdpHeader, EmptyMetadata>,
    ) -> Result<
        Box<Task>,
        (
            Packet<UdpHeader, EmptyMetadata>,
            Packet<UdpHeader, EmptyMetadata>,
        ),
    > {
        // First, parse the request packet.
        let req = req.parse_header::<PutRequest>();

        // Read fields off the request header.
        let mut tenant_id: TenantId = 0;
        let mut table_id: TableId = 0;
        let mut key_length = 0;
        let mut rpc_stamp = 0;

        {
            let hdr = req.get_header();
            tenant_id = hdr.common_header.tenant as TenantId;
            table_id = hdr.table_id as TableId;
            key_length = hdr.key_length;
            rpc_stamp = hdr.common_header.stamp;
        }

        // Next, write a header into the response packet.
        let mut res = res.push_header(&PutResponse::new(rpc_stamp))
            .expect("Failed to push PutResponse");

        // If the payload size is less than the key length, return an error.
        if req.get_payload().len() < key_length as usize {
            res.get_mut_header().common_header.status = RpcStatus::StatusMalformedRequest;
            return Err((
                req.deparse_header(PACKET_UDP_LEN as usize),
                res.deparse_header(PACKET_UDP_LEN as usize),
            ));
        }

        // Lookup the tenant, and get a handle to the allocator. Required to avoid capturing a
        // reference to Master in the generator below.
        let tenant = self.get_tenant(tenant_id);
        let alloc = self.heap.clone();

        // Create a generator for this request.
        let gen = Box::new(move || {
            let mut status: RpcStatus = RpcStatus::StatusTenantDoesNotExist;

            // If the tenant exists, check if it has a table with the given id,
            // and update the status of the rpc.
            let outcome = tenant.and_then(|tenant| {
                status = RpcStatus::StatusTableDoesNotExist;
                tenant.get_table(table_id)
            });

            // If the table exists, update the status of the rpc, and allocate an
            // object.
            if let Some(table) = outcome {
                // Get a reference to the key and value.
                status = RpcStatus::StatusMalformedRequest;
                let (key, val) = req.get_payload().split_at(key_length as usize);

                // If there is a value, then write it in.
                if val.len() > 0 {
                    status = RpcStatus::StatusInternalError;
                    let _result = alloc.object(tenant_id, table_id, key, val)
                                    // If the allocation succeeds, update the
                                    // status of the rpc, and insert the object
                                    // into the table.
                                    .and_then(| (key, obj) | {
                                        status = RpcStatus::StatusOk;
                                        table.put(key, obj);
                                        Some(())
                                    });
                }
            }

            // Update the response header.
            res.get_mut_header().common_header.status = status;

            // Deparse request and response packets to UDP, and return from the generator.
            return Some((
                req.deparse_header(PACKET_UDP_LEN as usize),
                res.deparse_header(PACKET_UDP_LEN as usize),
            ));

            // XXX: This yield is required to get the compiler to compile this closure into a
            // generator. It is unreachable and benign.
            yield 0;
        });

        // Create and return a native task.
        return Ok(Box::new(Native::new(TaskPriority::REQUEST, gen)));
    }

    /// Handles the invoke RPC request.
    ///
    /// If issued by a valid tenant for a valid extension, invokes the extension.
    ///
    /// # Arguments
    ///
    /// * `req`: The RPC request packet sent by the client, parsed upto it's UDP header.
    /// * `res`: The RPC response packet, with pre-allocated headers upto UDP.
    ///
    /// # Return
    ///
    /// A Container task that can be scheduled by the database. In the case of an error, the passed
    /// in request and response packets are returned with the response status appropriately set.
    #[allow(unused_assignments)]
    fn invoke(
        &self,
        req: Packet<UdpHeader, EmptyMetadata>,
        res: Packet<UdpHeader, EmptyMetadata>,
    ) -> Result<
        Box<Task>,
        (
            Packet<UdpHeader, EmptyMetadata>,
            Packet<UdpHeader, EmptyMetadata>,
        ),
    > {
        // First, parse the request packet.
        let req = req.parse_header::<InvokeRequest>();

        // Read fields of the request header.
        let mut tenant_id: TenantId = 0;
        let mut name_length: usize = 0;
        let mut args_length: usize = 0;
        let mut rpc_stamp = 0;

        {
            let hdr = req.get_header();
            tenant_id = hdr.common_header.tenant as TenantId;
            name_length = hdr.name_length as usize;
            args_length = hdr.args_length as usize;
            rpc_stamp = hdr.common_header.stamp;
        }

        // Next, add a header to the response packet.
        let mut res = res.push_header(&InvokeResponse::new(rpc_stamp))
            .expect("Failed to push InvokeResponse");

        // If the payload size is less than the sum of the name and args
        // length, return an error.
        if req.get_payload().len() < name_length + args_length {
            res.get_mut_header().common_header.status = RpcStatus::StatusMalformedRequest;
            return Err((
                req.deparse_header(PACKET_UDP_LEN as usize),
                res.deparse_header(PACKET_UDP_LEN as usize),
            ));
        }

        // Read the extension's name from the request payload.
        let mut name = Vec::new();
        name.extend_from_slice(req.get_payload().split_at(name_length).0);
        let name: String = String::from_utf8(name).expect("ERROR: Failed to get ext name.");

        let mut status = RpcStatus::StatusTenantDoesNotExist;

        // Check if the request was issued by a valid tenant.
        if let Some(tenant) = self.get_tenant(tenant_id) {
            // If the tenant is valid, check if the extension exists inside the database after
            // setting the RPC status appropriately.
            status = RpcStatus::StatusInvalidExtension;
            if let Some(ext) = self.extensions.get(tenant_id, &name) {
                let db = Rc::new(Context::new(
                    req,
                    name_length,
                    args_length,
                    res,
                    tenant,
                    Arc::clone(&self.heap),
                ));

                return Ok(Box::new(Container::new(TaskPriority::REQUEST, db, ext)));
            }
        }

        // A Task could not be created. Set the status of the RPC and return.
        res.get_mut_header().common_header.status = status;

        return Err((
            req.deparse_header(PACKET_UDP_LEN as usize),
            res.deparse_header(PACKET_UDP_LEN as usize),
        ));
    }
}

/// Implementation of the Service trait for Master, allowing it to service RPC requests.
impl Service for Master {
    /// Lookup the Service trait for documentation.
    fn dispatch(
        &self,
        op: OpCode,
        req: Packet<UdpHeader, EmptyMetadata>,
        res: Packet<UdpHeader, EmptyMetadata>,
    ) -> Result<
        Box<Task>,
        (
            Packet<UdpHeader, EmptyMetadata>,
            Packet<UdpHeader, EmptyMetadata>,
        ),
    > {
        // Based on the opcode, call the relevant RPC handler.
        match op {
            OpCode::SandstormGetRpc => {
                return self.get(req, res);
            }

            OpCode::SandstormPutRpc => {
                return self.put(req, res);
            }

            OpCode::SandstormInvokeRpc => {
                return self.invoke(req, res);
            }

            _ => {
                return Err((req, res));
            }
        }
    }
}
