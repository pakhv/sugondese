use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Route(pub HashMap<String, String>);

#[derive(Debug, PartialEq, Eq)]
pub struct Query(pub HashMap<String, String>);
