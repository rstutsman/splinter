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

use std::sync::Arc;
use std::mem::size_of;
use std::cell::RefCell;

use super::tenant::Tenant;
use super::alloc::Allocator;
use super::wireformat::{InvokeRequest, InvokeResponse};

use sandstorm::db::DB;
use sandstorm::buf::{ReadBuf, WriteBuf};

use e2d2::common::EmptyMetadata;
use e2d2::headers::{IpHeader, MacHeader, UdpHeader};
use e2d2::interface::{Packet, packet_from_mbuf_no_increment};

/// This type is passed into the init method of every extension. The methods
/// on this type form the interface allowing extensions to read and write
/// data from and to the database. The constructors for this type (new() and
/// default()) should be exposed only to trusted code, and not to extensions.
pub struct Context {
    // The packet/buffer consisting of the RPC request header and payload
    // that invoked the extension. This is required to potentially pass in
    // arguments to an extension. For example, a get() extension might require
    // a key and table identifier to be passed in.
    request: Packet<InvokeRequest, EmptyMetadata>,

    // The offset inside the request packet/buffer's payload at which the
    // arguments to the extension begin.
    args_offset: usize,

    // The total length of the extension's arguments that were written into the
    // request packet/buffer's payload.
    args_length: usize,

    // A pre-populated RPC response packet/buffer for the invoked extension.
    // This is required because the extension might need to return something
    // to the issuing client/tenant. For example, a get() extension will need
    // to return a value to the issuing client/tenant.
    response: RefCell<Packet<InvokeResponse, EmptyMetadata>>,

    // The tenant that invoked this extension. Required to access the tenant's
    // data, and potentially for accounting.
    tenant: Arc<Tenant>,

    // The allocator that will be used to allow the extension to write data to
    // one of it's tables.
    heap: Arc<Allocator>,
}

// Methods on Context.
impl Context {
    /// This function returns a context that can be used to invoke an extension.
    ///
    /// # Arguments
    ///
    /// * `req`:      The invoke() RPC request packet/buffer consisting of the
    ///               header and payload.
    /// * `args_off`: The offset into the payload of `req` at which the
    ///               extension's arguments begin.
    /// * `args_len`: The length of the extension's arguments that were written
    ///               into the payload of `req`.
    /// * `res`:      A pre-allocated RPC response packet/buffer consisting of a
    ///               response header for the invoke() request.
    /// * `tenant`:   An `Arc` to the tenant that issued the invoke() request.
    /// * `alloc`:    An `Arc` to the memory allocator. Required to allow the
    ///               extension to issue writes to the database.
    ///
    /// # Result
    /// A context that can be used to invoke an extension.
    pub fn new(req: Packet<InvokeRequest, EmptyMetadata>,
               args_off: usize, args_len: usize,
               res: Packet<InvokeResponse, EmptyMetadata>,
               tenant: Arc<Tenant>, alloc: Arc<Allocator>)
               -> Context
    {
        Context {
            request: req,
            args_offset: args_off,
            args_length: args_len,
            response: RefCell::new(res),
            tenant: tenant,
            heap: alloc,
        }
    }

    /// This method commits any changes made by an extension to the database.
    /// It consumes the context, and returns the request and response
    /// packets/buffers to the caller.
    ///
    /// # Return
    /// A tupule whose first member is the request packet/buffer for the
    /// extension, and whose second member is the response packet/buffer
    /// that can be sent back to the tenant.
    pub unsafe fn commit(self) -> (Packet<InvokeRequest, EmptyMetadata>,
                                   Packet<InvokeResponse, EmptyMetadata>)
    {
        // Rewrap the request into a new packet.
        let req = packet_from_mbuf_no_increment::<InvokeRequest>(
                    self.request.get_mbuf(), size_of::<MacHeader>() +
                    size_of::<IpHeader>() + size_of::<UdpHeader>() +
                    size_of::<InvokeRequest>());

        // Rewrap the response into a new packet.
        let res = packet_from_mbuf_no_increment::<InvokeResponse>(
                    self.response.into_inner().get_mbuf(),
                    size_of::<MacHeader>() + size_of::<IpHeader>() +
                    size_of::<UdpHeader>() + size_of::<InvokeResponse>());

        // Return the request and response packet. At this point, the context
        // is dropped, and can never be used again.
        return (req, res);
    }
}

// The DB trait for Context.
impl DB for Context {
    /// Lookup the `DB` trait for documentation on this method.
    fn get(&self, table_id: u64, key: &[u8]) -> Option<ReadBuf> {
        // Lookup the database for the key value pair. If it exists, then update
        // the read set and return the value.
        self.tenant.get_table(table_id)
                    .and_then(| table | { table.get(key) })
                    // The object exists in the database. Get a handle to it's
                    // key and value.
                    .and_then(| object | { self.heap.resolve(object) })
                    // Return the value wrapped up inside a safe type.
                    .and_then(| (_k, v) | { unsafe { Some(ReadBuf::new(v)) } })
    }

    /// Lookup the `DB` trait for documentation on this method.
    fn alloc(&self, table_id: u64, key: &[u8], val_len: u64)
             -> Option<WriteBuf>
    {
        // Check if the tenant owns a table with the requested identifier.
        // If it does, perform and return an allocation.
        self.tenant.get_table(table_id)
                    .and_then(| _table | { self.heap.raw(self.tenant.id(),
                                                         table_id,
                                                         key, val_len) })
                    .and_then(| buf | {
                        unsafe { Some(WriteBuf::new(table_id, buf)) } })
    }

    /// Lookup the `DB` trait for documentation on this method.
    fn put(&self, buf: WriteBuf) -> bool {
        // Convert the passed in Writebuf to read only.
        let (table_id, buf) = unsafe { buf.freeze() };

        // If the table exists, write to the database.
        if let Some(table) = self.tenant.get_table(table_id) {
            return self.heap.resolve(buf.clone())
                            .map_or(false, | (k, _v) | {
                                table.put(k, buf);
                                true });
        }

        return false;
    }

    /// Lookup the `DB` trait for documentation on this method.
    fn args(&self) -> &[u8] {
        // Return a slice to the arguments off the request packet/buffer's
        // payload.
        self.request.get_payload()
                    .split_at(self.args_offset).1
                    .split_at(self.args_length).0
    }

    /// Lookup the `DB` trait for documentation on this method.
    fn resp(&self, data: &[u8]) {
        // Write the passed in data to the response packet/buffer.
        self.response.borrow_mut()
                        .add_to_payload_tail(data.len(), data)
                        .unwrap();
    }

    /// Lookup the `DB` trait for documentation on this method.
    fn debug_log(&self, _msg: &str) {
        ;
    }
}