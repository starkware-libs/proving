//! Binary tree shape driving the witness for the Merkle commitment circuit (see
//! [`crate::unpacker`]).

/// A full binary tree, arbitrary and possibly unbalanced. Each `Leaf` carries an `L` (in the
/// unpacker, a [`Node`](crate::unpacker::Node)) and each `Internal` node an `N` (its supplied
/// `circuit_hash`; its `subtree_hash` is *computed* from its children, so it is not carried here)
/// plus its two children.
#[derive(Clone)]
pub enum BinaryTree<L, N> {
    Leaf(L),
    Internal(N, Box<[BinaryTree<L, N>; 2]>),
}

impl<L, N> BinaryTree<L, N> {
    /// Every leaf value in left-to-right (depth-first) order.
    ///
    /// The sequence follows the tree's *shape*, so two trees with the same leaf multiset may yield
    /// different sequences. The unpacker relies only on this being the same traversal order its own
    /// recursion uses.
    pub fn leaves(&self) -> Vec<&L> {
        let mut out = Vec::new();
        self.collect_leaves(&mut out);
        out
    }

    fn collect_leaves<'a>(&'a self, out: &mut Vec<&'a L>) {
        match self {
            BinaryTree::Leaf(v) => out.push(v),
            BinaryTree::Internal(_, children) => {
                children[0].collect_leaves(out);
                children[1].collect_leaves(out);
            }
        }
    }
}
