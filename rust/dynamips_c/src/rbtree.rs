//! Red/Black Trees.

use crate::prelude::*;

/// Comparison function for 2 keys
pub type tree_fcompare = Option<unsafe extern "C" fn(key1: *mut c_void, key2: *mut c_void, opt: *mut c_void) -> c_int>;

/// User function to call when using rbtree_foreach
pub type tree_fforeach = Option<unsafe extern "C" fn(key: *mut c_void, value: *mut c_void, opt: *mut c_void)>;

// Node colors // TODO enum
pub const RBTREE_RED: c_short = 0;
pub const RBTREE_BLACK: c_short = 1;

/// Description of a node in a Red/Black tree. To be more efficient, keys are
/// stored with a void * pointer, allowing to use different type keys.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rbtree_node {
    /// Key and Value
    pub key: *mut c_void,
    pub value: *mut c_void,
    /// Left and right nodes
    pub left: *mut rbtree_node,
    pub right: *mut rbtree_node,
    /// Parent node
    pub parent: *mut rbtree_node,
    /// Node color
    pub color: c_short,
}

#[no_mangle]
pub extern "C" fn _export(_: tree_fcompare, _: tree_fforeach, _: *mut rbtree_node) {}
