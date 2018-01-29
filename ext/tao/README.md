# Notes: TAO implementation in Rust.
This is a collection of my notes and thoughts as I learn rust and implement tao. This is also a store of my questions and answers to those questions.


Helpful TAO links I've found:
*  [TAO: The power of the graph](https://www.facebook.com/notes/facebook-engineering/tao-the-power-of-the-graph/10151525983993920/ "Facebook TAO").   


## Terminology
Node: **Object** (data item).  
Edge: **Association** (relationship between two Objects).

#### Association Queries:
**assoc_create**: (id1, association_type, id2, time, bag_of_bytes?)  
**assoc_change_type/update**: (id1, association_type, id2, new_type)  
   ?: Why would we want to change type? Seems kind of pointless...   
**assoc_delete**: (id1, association_type, id2)  

#### Association List Queries:
**Assocation List**: formed from a pair of a particular association_type and object. Each object may have multiple association lists.  
**assoc_retrieve**: (id1, association_type, id2set) retrieve associations specified by this triplet.  
**assoc_count**: (id1, association_type) reports number of edges of the given type associated with object id1. (Basically the size of the id2 set that would be returned by a retrieve call.)

#### Object Queries:
**object_create**: (kv_info)  
**object_retrieve**: (id)  
**object_update**: (id, kv_info)  
**object_delete**: (id)  

## Other Unorganized Requirements
* When one edge of the graph is deleted/created, its inverse edge (if one exists) will be automatically deleted/created.
* Each object will at least contain basic CRUD (create, retrieve, update, delete) methods.
* All objects of the same type will have the same fields.
* New fields can be added to an object at any time.

   **?:** Does this work like an interface? There's some sort of object-type template that each object follows and that template can be changed?
   **?** If I make a change to an object-type does that change get made in every existing instance of that object-type?
* Old fields can be marked as deprecated by editing the object-type schema.
* Every _association_ will have a time attribute (used to represent the creation time of the _association_ to promote _creation-time locality_).
* \* Reports _count_ in constant time.
* Association query results are ordered by time.
* All Objects of all types will have a unique id.
* Only one association of a given type between two object can exist.
* Associations model actions.


##### Questions
* What do the tables in the DB look like?

  **A:** Two column tables, first one labeled key, next one labeled value.

* I'm having a difficult time understanding uses and motivation.

  **A:** Think shared resources, BYODB (Build your own DB), speed, and Key-Value Store

* What's the deal with the key value pairs? I'm having difficult time understanding what these key-value pairs are and why they are/aren't important.

  **A:** This is not a DB like I've been introduced to. It's an entirely different subset of Datastore than a SQL DB. It's literally just a giant non-volatile HashMap where people can store organized/unorganized chunks of data. Nothing fancy, nothing crazy.
