use std::{collections::{HashMap, HashSet}, sync::OnceLock};

use daggy::{petgraph::{algo::dijkstra, visit::IntoNodeIdentifiers}, Dag, NodeIndex};

use crate::{Associativity, Error, Result};

/// Stores context for parsing.
/// 
/// Each `Context` instance contains a partial ordering of operator precedences,
/// a list of associativities of operators with themselves, and a set containing
/// all expressions that are infix by default.
/// 
/// The context stores static information about parsing.
/// Cheap, copyable, and dynamic information is stored in the [Reader] instance.
pub struct Context {
    partial_order: Dag<String, (), u32>,
    self_referential: HashMap<String, Associativity>,
    infix: HashSet<String>,
}

impl Context {

    pub fn standard() -> &'static Self {
        static LOCK: OnceLock<Context> = OnceLock::new();
        LOCK.get_or_init(|| {
            let mut context = Context::new();

            // ["->", "^", "*"]
            //     .iter().for_each(|i| context.ra(i));
    
            context.gt("a", "@");
            context.gt("a", "-");
            // context.lt(",", "@");
            context.gt(".", "-");
            context.gt("@", ".");
    
            ["."]
                .iter().for_each(|i| context.infix(i));
    
            context
        })
    }

    pub fn new() -> Self {
        Context {
            partial_order: Dag::new(),
            self_referential: HashMap::new(),
            infix: HashSet::new()
        }
    }

    /// Returns the numerical index, in the partial order graph, of the provided
    /// operator, or `None` if none exists.
    pub fn get_ident(&self, operator: &str) -> Option<NodeIndex> {
        self.partial_order.node_identifiers()
            .find(|i| self.partial_order[*i] == operator)
    }

    /// Returns the numerical index, in the partial order graph, of the provided
    /// operator, creating one if necessary.
    pub fn ensure_ident(&mut self, operator: &str) -> NodeIndex {
        self.get_ident(operator)
            .unwrap_or_else(|| self.partial_order.add_node(operator.to_owned()))
    }

    /// Returns whether or not there is a path from the left node to the right
    /// node using Dijkstra's algorithm.
    pub fn path_from(&self, left: NodeIndex, right: NodeIndex) -> bool {
        dijkstra(&self.partial_order, left, Some(right), |_| 1).contains_key(&right)
    }

    /// Indicates that the first operator has higher binding power than the
    /// second by adding an edge from the first to the second.
    /// 
    /// If there is already a path from the second operator to the first
    /// operator, a cycle would occur; in this case, the edge is not added and
    /// `false` is returned.
    pub fn gt(&mut self, first: &str, second: &str) -> bool {
        let f = self.ensure_ident(first);
        let s = self.ensure_ident(second);

        self.partial_order.update_edge(f, s, ()).is_ok()
    }

    /// Indicates that the second operator has higher binding power than the
    /// first by adding an edge from the second to the first.
    /// 
    /// This has identical semantics to [Context::gt], except with the arguments
    /// switched.
    pub fn lt(&mut self, first: &str, second: &str) -> bool {
        self.gt(second, first)
    }

    /// Gets the defined associativity between two operators.
    /// If they're the same operator, it queries a local map to see what to
    /// return.
    /// 
    /// If they're not, it refers to the internal directed acyclic graph.
    /// If a path exists from left -> right, left binds stronger than right, so
    /// we use left associativity. If a path exists from right -> left, then
    /// right binds stronger than left, so we use right associativity.
    /// The acyclicity of the graph ensures these conditions are mutually
    /// exclusive.
    pub fn get_defined_associativity(&self, left: &str, right: &str) -> Option<Associativity> {
        if left == right {
            return self.self_referential.get(left).copied();
        }
        let (infix_l, infix_r) = (self.get_ident(left)?, self.get_ident(right)?);
        
        if self.path_from(infix_l, infix_r) {
            Some(Associativity::Left)
        } else if self.path_from(infix_r, infix_l) {
            Some(Associativity::Right)
        } else {
            None
        }
    }

    /// Similar behaviour to Parser::get_defined_associativity except with
    /// default values: prefix expressions bind stronger than infix expressions,
    /// prefix operators bind to the left by default, and infix expressions have
    /// no default associativity.
    pub fn get_associativity(&self, left: &str, right: &str, input: &&str) -> Result<Associativity> {
        if let Some(defined) = self.get_defined_associativity(left, right) {
            return Ok(defined);
        }

        match (self.is_infix(left), self.is_infix(right)) {
            (true, false) => Ok(Associativity::Left),
            (false, true) => Ok(Associativity::Right),
            (false, false) => Ok(Associativity::Left),
            (true, true) => Err((input.to_string(), Error::UndefinedAssociativity(left.into(), right.into()))),
        }
    }

    /// Marks the provided operator as left associative (to itself).
    pub fn la(&mut self, op: &str) {
        self.self_referential.insert(op.to_owned(), Associativity::Left);
    }

    /// Marks the provided operator as right associative (to itself).
    pub fn ra(&mut self, op: &str) {
        self.self_referential.insert(op.to_owned(), Associativity::Right);
    }

    /// Marks the provided operator as an infix operator.
    pub fn infix(&mut self, op: &str) {
        self.infix.insert(op.to_owned());
    }

    /// Determines whether or not a given token is an infix operator.
    pub fn is_infix(&self, op: &str) -> bool {
        self.infix.contains(op)
    }
    
}