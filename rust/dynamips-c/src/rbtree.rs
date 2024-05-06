//! IPFlow Collector/Dynamips
//! Copyright (c) 2004 Christophe Fillot.
//! E-mail: cf@utc.fr
//!
//! rbtree.c: Red/Black Trees.
//! Red/Black Trees.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::mempool::*;
use std::cmp::max;

/// Comparison function for 2 keys
pub type tree_fcompare = Option<unsafe extern "C" fn(key1: *mut c_void, key2: *mut c_void, opt: *mut c_void) -> c_int>;

/// User function to call when using rbtree_foreach
pub type tree_fforeach = Option<unsafe extern "C" fn(key: *mut c_void, value: *mut c_void, opt: *mut c_void)>;

/// Node colors // TODO enum
pub const RBTREE_RED: c_int = 0;
pub const RBTREE_BLACK: c_int = 1;

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

/// Description of a Red/Black tree. For commodity, a name can be given to the
/// tree. "rbtree_comp" is a pointer to a function, defined by user, which
/// compares keys during node operations.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct rbtree_tree {
    pub node_count: c_int,      // Number of Nodes
    pub mp: mempool_t,          // Memory pool
    pub nil: rbtree_node,       // Sentinel
    pub root: *mut rbtree_node, // Root node
    pub key_cmp: tree_fcompare, // Key comparison function
    pub opt_data: *mut c_void,  // Optional data for comparison
}

unsafe fn rbtree_nil(tree: *mut rbtree_tree) -> *mut rbtree_node {
    addr_of_mut!((*tree).nil)
}
unsafe fn NIL(tree: *mut rbtree_tree, x: *mut rbtree_node) -> bool {
    x == rbtree_nil(tree) || x.is_null()
}

/// Allocate memory for a new node
unsafe fn rbtree_node_alloc(tree: *mut rbtree_tree, key: *mut c_void, value: *mut c_void) -> *mut rbtree_node {
    let node: *mut rbtree_node = mp_alloc_n0(addr_of_mut!((*tree).mp), size_of::<rbtree_node>()).cast::<_>();
    if node.is_null() {
        return null_mut();
    }

    (*node).key = key;
    (*node).value = value;
    (*node).left = rbtree_nil(tree);
    (*node).right = rbtree_nil(tree);
    (*node).parent = rbtree_nil(tree);
    (*node).color = -1;
    node
}

/// Free memory used by a node
#[inline]
unsafe fn rbtree_node_free(_tree: *mut rbtree_tree, node: *mut rbtree_node) {
    mp_free(node.cast::<_>());
}

/// Returns the node which represents the minimum value
#[inline]
unsafe fn rbtree_min(tree: *mut rbtree_tree, mut x: *mut rbtree_node) -> *mut rbtree_node {
    while !NIL(tree, (*x).left) {
        x = (*x).left;
    }

    x
}

/// Returns the node which represents the maximum value
#[inline]
unsafe fn rbtree_max(tree: *mut rbtree_tree, mut x: *mut rbtree_node) -> *mut rbtree_node {
    while !NIL(tree, (*x).right) {
        x = (*x).right;
    }

    x
}

/// Returns the successor of a node
#[inline]
unsafe fn rbtree_successor(tree: *mut rbtree_tree, mut x: *mut rbtree_node) -> *mut rbtree_node {
    if !NIL(tree, (*x).right) {
        return rbtree_min(tree, (*x).right);
    }

    let mut y: *mut rbtree_node = (*x).parent;
    while !NIL(tree, y) && x == (*y).right {
        x = y;
        y = (*y).parent;
    }

    y
}

/// Left rotation
#[inline]
unsafe fn rbtree_left_rotate(tree: *mut rbtree_tree, x: *mut rbtree_node) {
    let y: *mut rbtree_node = (*x).right;
    (*x).right = (*y).left;

    if !NIL(tree, (*x).right) {
        (*(*x).right).parent = x;
    }

    (*y).parent = (*x).parent;

    if NIL(tree, (*x).parent) {
        (*tree).root = y;
    } else if x == (*(*x).parent).left {
        (*(*x).parent).left = y;
    } else {
        (*(*x).parent).right = y;
    }

    (*y).left = x;
    (*x).parent = y;
}

/// Right rotation
#[inline]
unsafe fn rbtree_right_rotate(tree: *mut rbtree_tree, y: *mut rbtree_node) {
    let x: *mut rbtree_node = (*y).left;
    (*y).left = (*x).right;

    if !NIL(tree, (*y).left) {
        (*(*y).left).parent = y;
    }

    (*x).parent = (*y).parent;

    if NIL(tree, (*y).parent) {
        (*tree).root = x;
    } else if (*(*y).parent).left == y {
        (*(*y).parent).left = x;
    } else {
        (*(*y).parent).right = x;
    }

    (*x).right = y;
    (*y).parent = x;
}

/// insert a new node
unsafe fn rbtree_insert_new(tree: *mut rbtree_tree, key: *mut c_void, value: *mut c_void, exists: *mut c_int) -> *mut rbtree_node {
    let mut nodeplace: *mut *mut rbtree_node = addr_of_mut!((*tree).root);
    let mut parent: *mut rbtree_node = null_mut();
    *exists = FALSE;

    loop {
        let node: *mut rbtree_node = *nodeplace;

        if NIL(tree, node) {
            break;
        }

        let comp: c_int = (*tree).key_cmp.unwrap()(key, (*node).key, (*tree).opt_data);

        if comp == 0 {
            *exists = TRUE;
            (*node).value = value;
            return node;
        }

        parent = node;
        nodeplace = if comp > 0 { addr_of_mut!((*node).right) } else { addr_of_mut!((*node).left) };
    }

    // create a new node
    let new_node: *mut rbtree_node = rbtree_node_alloc(tree, key, value);
    if new_node.is_null() {
        return null_mut();
    }

    *nodeplace = new_node;
    (*new_node).parent = parent;

    (*tree).node_count += 1;
    new_node
}

/// Insert a node in a Red/Black Tree
#[no_mangle]
pub unsafe extern "C" fn rbtree_insert(tree: *mut rbtree_tree, key: *mut c_void, value: *mut c_void) -> c_int {
    // insert a new node (if necessary)
    let mut exists: c_int = 0;
    let mut x: *mut rbtree_node = rbtree_insert_new(tree, key, value, addr_of_mut!(exists));

    if exists != 0 {
        return 0;
    }
    if x.is_null() {
        return -1;
    }

    (*tree).node_count += 1;

    // maintains red-black properties
    (*x).color = RBTREE_RED as c_short;

    while x != (*tree).root && (*(*x).parent).color as c_int == RBTREE_RED {
        if (*x).parent == (*(*(*x).parent).parent).left {
            let y: *mut rbtree_node = (*(*(*x).parent).parent).right;

            if (*y).color as c_int == RBTREE_RED {
                (*(*x).parent).color = RBTREE_BLACK as c_short;
                (*y).color = RBTREE_BLACK as c_short;
                (*(*(*x).parent).parent).color = RBTREE_RED as c_short;
                x = (*(*x).parent).parent;
            } else {
                if x == (*(*x).parent).right {
                    x = (*x).parent;
                    rbtree_left_rotate(tree, x);
                }

                (*(*x).parent).color = RBTREE_BLACK as c_short;
                (*(*(*x).parent).parent).color = RBTREE_RED as c_short;
                rbtree_right_rotate(tree, (*(*x).parent).parent);
            }
        } else {
            let y: *mut rbtree_node = (*(*(*x).parent).parent).left;

            if (*y).color as c_int == RBTREE_RED {
                (*(*x).parent).color = RBTREE_BLACK as c_short;
                (*y).color = RBTREE_BLACK as c_short;
                (*(*(*x).parent).parent).color = RBTREE_RED as c_short;
                x = (*(*x).parent).parent;
            } else {
                if x == (*(*x).parent).left {
                    x = (*x).parent;
                    rbtree_right_rotate(tree, x);
                }

                (*(*x).parent).color = RBTREE_BLACK as c_short;
                (*(*(*x).parent).parent).color = RBTREE_RED as c_short;
                rbtree_left_rotate(tree, (*(*x).parent).parent);
            }
        }
    }

    (*(*tree).root).color = RBTREE_BLACK as c_short;
    0
}

/// Lookup for a node corresponding to "key"
#[inline]
unsafe fn rbtree_lookup_node(tree: *mut rbtree_tree, key: *mut c_void) -> *mut rbtree_node {
    let mut node: *mut rbtree_node = (*tree).root;

    loop {
        if NIL(tree, node) {
            break; // key not found
        }

        let comp: c_int = (*tree).key_cmp.unwrap()(key, (*node).key, (*tree).opt_data);
        if comp == 0 {
            break; // exact match
        }

        node = if comp > 0 { (*node).right } else { (*node).left };
    }

    node
}

/// Lookup for a node corresponding to "key". If node does not exist,
/// function returns null pointer.
#[no_mangle]
pub unsafe extern "C" fn rbtree_lookup(tree: *mut rbtree_tree, key: *mut c_void) -> *mut c_void {
    (*rbtree_lookup_node(tree, key)).value
}

/// Restore Red/black tree properties after a removal
unsafe fn rbtree_removal_fixup(tree: *mut rbtree_tree, mut x: *mut rbtree_node) {
    while x != (*tree).root && (*x).color as c_int == RBTREE_BLACK {
        if x == (*(*x).parent).left {
            let mut w: *mut rbtree_node = (*(*x).parent).right;

            if (*w).color as c_int == RBTREE_RED {
                (*w).color = RBTREE_BLACK as c_short;
                (*(*x).parent).color = RBTREE_RED as c_short;
                rbtree_left_rotate(tree, (*x).parent);
                w = (*(*x).parent).right;
            }

            if (*(*w).left).color as c_int == RBTREE_BLACK && (*(*w).right).color as c_int == RBTREE_BLACK {
                (*w).color = RBTREE_RED as c_short;
                x = (*x).parent;
            } else {
                if (*(*w).right).color as c_int == RBTREE_BLACK {
                    (*(*w).left).color = RBTREE_BLACK as c_short;
                    (*w).color = RBTREE_RED as c_short;
                    rbtree_right_rotate(tree, w);
                    w = (*(*x).parent).right;
                }

                (*w).color = (*(*x).parent).color;
                (*(*x).parent).color = RBTREE_BLACK as c_short;
                (*(*w).right).color = RBTREE_BLACK as c_short;
                rbtree_left_rotate(tree, (*x).parent);
                x = (*tree).root;
            }
        } else {
            let mut w: *mut rbtree_node = (*(*x).parent).left;

            if (*w).color as c_int == RBTREE_RED {
                (*w).color = RBTREE_BLACK as c_short;
                (*(*x).parent).color = RBTREE_RED as c_short;
                rbtree_right_rotate(tree, (*x).parent);
                w = (*(*x).parent).left;
            }

            if (*(*w).right).color as c_int == RBTREE_BLACK && (*(*w).left).color as c_int == RBTREE_BLACK {
                (*w).color = RBTREE_RED as c_short;
                x = (*x).parent;
            } else {
                if (*(*w).left).color as c_int == RBTREE_BLACK {
                    (*(*w).right).color = RBTREE_BLACK as c_short;
                    (*w).color = RBTREE_RED as c_short;
                    rbtree_left_rotate(tree, w);
                    w = (*(*x).parent).left;
                }

                (*w).color = (*(*x).parent).color;
                (*(*x).parent).color = RBTREE_BLACK as c_short;
                (*(*w).left).color = RBTREE_BLACK as c_short;
                rbtree_right_rotate(tree, (*x).parent);
                x = (*tree).root;
            }
        }
    }

    (*x).color = RBTREE_BLACK as c_short;
}

/// Removes a node out of a tree
#[no_mangle]
pub unsafe extern "C" fn rbtree_remove(tree: *mut rbtree_tree, key: *mut c_void) -> *mut c_void {
    let z: *mut rbtree_node = rbtree_lookup_node(tree, key);

    if NIL(tree, z) {
        return null_mut();
    }

    let value: *mut c_void = (*z).value;

    let y: *mut rbtree_node = if NIL(tree, (*z).left) || NIL(tree, (*z).right) { z } else { rbtree_successor(tree, z) };

    let x: *mut rbtree_node = if !NIL(tree, (*y).left) { (*y).left } else { (*y).right };

    (*x).parent = (*y).parent;

    if NIL(tree, (*y).parent) {
        (*tree).root = x;
    } else if y == (*(*y).parent).left {
        (*(*y).parent).left = x;
    } else {
        (*(*y).parent).right = x;
    }

    if y != z {
        (*z).key = (*y).key;
        (*z).value = (*y).value;
    }

    if (*y).color as c_int == RBTREE_BLACK {
        rbtree_removal_fixup(tree, x);
    }

    rbtree_node_free(tree, y);
    (*tree).node_count += 1;
    value
}

unsafe fn rbtree_foreach_node(tree: *mut rbtree_tree, node: *mut rbtree_node, user_fn: tree_fforeach, opt: *mut c_void) {
    if !NIL(tree, node) {
        rbtree_foreach_node(tree, (*node).left, user_fn, opt);
        user_fn.unwrap()((*node).key, (*node).value, opt);
        rbtree_foreach_node(tree, (*node).right, user_fn, opt);
    }
}

/// Call the specified function for each node
#[no_mangle]
pub unsafe extern "C" fn rbtree_foreach(tree: *mut rbtree_tree, user_fn: tree_fforeach, opt: *mut c_void) -> c_int {
    if tree.is_null() {
        return -1;
    }

    rbtree_foreach_node(tree, (*tree).root, user_fn, opt);
    0
}

/// Returns the maximum height of the right and left sub-trees
unsafe fn rbtree_height_node(tree: *mut rbtree_tree, node: *mut rbtree_node) -> c_int {
    let lh: c_int = if !NIL(tree, (*node).left) { rbtree_height_node(tree, (*node).left) } else { 0 };
    let rh: c_int = if !NIL(tree, (*node).right) { rbtree_height_node(tree, (*node).right) } else { 0 };
    1 + max(lh, rh)
}

/// Compute the height of a Red/Black tree
#[no_mangle]
pub unsafe extern "C" fn rbtree_height(tree: *mut rbtree_tree) -> c_int {
    if !NIL(tree, (*tree).root) {
        rbtree_height_node(tree, (*tree).root)
    } else {
        0
    }
}

/// Returns the number of nodes
#[no_mangle]
pub unsafe extern "C" fn rbtree_node_count(tree: *mut rbtree_tree) -> c_int {
    (*tree).node_count
}

/// Purge all nodes
#[no_mangle]
pub unsafe extern "C" fn rbtree_purge(tree: *mut rbtree_tree) {
    mp_free_all_blocks(addr_of_mut!((*tree).mp));
    (*tree).node_count = 0;

    // just in case
    libc::memset(rbtree_nil(tree).cast::<_>(), 0, size_of::<rbtree_node>());
    (*rbtree_nil(tree)).color = RBTREE_BLACK as c_short;

    // reset root
    (*tree).root = rbtree_nil(tree);
}

/// Check a node
unsafe fn rbtree_check_node(tree: *mut rbtree_tree, node: *mut rbtree_node) -> c_int {
    if !NIL(tree, node) {
        return 0;
    }

    if !NIL(tree, (*node).left) {
        if (*tree).key_cmp.unwrap()((*node).key, (*(*node).left).key, (*tree).opt_data) <= 0 {
            return -1;
        }

        if rbtree_check_node(tree, (*node).left) == -1 {
            return -1;
        }
    }

    if !NIL(tree, (*node).right) {
        if (*tree).key_cmp.unwrap()((*node).key, (*(*node).right).key, (*tree).opt_data) >= 0 {
            return -1;
        }

        if rbtree_check_node(tree, (*node).right) == -1 {
            return -1;
        }
    }

    0
}

/// Check tree consistency
#[no_mangle]
pub unsafe extern "C" fn rbtree_check(tree: *mut rbtree_tree) -> c_int {
    rbtree_check_node(tree, (*tree).root)
}

/// Create a new Red/Black tree
#[no_mangle]
pub unsafe extern "C" fn rbtree_create(key_cmp: tree_fcompare, opt_data: *mut c_void) -> *mut rbtree_tree {
    let tree: *mut rbtree_tree = libc::malloc(size_of::<rbtree_tree>()).cast::<_>();
    if tree.is_null() {
        return null_mut();
    }

    libc::memset(tree.cast::<_>(), 0, size_of::<rbtree_tree>());

    // initialize the memory pool
    if mp_create_fixed_pool(addr_of_mut!((*tree).mp), cstr!("Red-Black Tree")).is_null() {
        libc::free(tree.cast::<_>());
        return null_mut();
    }

    // initialize the "nil" pointer
    libc::memset(rbtree_nil(tree).cast::<_>(), 0, size_of::<rbtree_node>());
    (*rbtree_nil(tree)).color = RBTREE_BLACK as c_short;

    (*tree).key_cmp = key_cmp;
    (*tree).opt_data = opt_data;
    (*tree).root = rbtree_nil(tree);
    tree
}

/// Delete a Red/Black tree
#[no_mangle]
pub unsafe extern "C" fn rbtree_delete(tree: *mut rbtree_tree) {
    if !tree.is_null() {
        mp_free_pool(addr_of_mut!((*tree).mp));
        libc::free(tree.cast::<_>());
    }
}
