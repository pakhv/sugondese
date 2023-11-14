use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Route(pub HashMap<String, String>);

#[derive(Debug, PartialEq)]
pub struct Query(pub HashMap<String, String>);
