
#[macro_export]
macro_rules! get_node {
    ($typ:ty, $name:literal) => {
        ::paste::paste! {
            fn [<get_node_ $name:snake>] (&self) -> Gd<$typ> {
                self.base().get_node_as($name)
            }
        }
    };
}

#[macro_export]
macro_rules! get_node_named {
    ($typ:ty, $node_name:literal, $function_name:literal) => {
        ::paste::paste! {
            fn [<get_node_ $function_name>] (&self) -> Gd<$typ> {
                self.base().get_node_as($node_name)
            }
        }
    };
}