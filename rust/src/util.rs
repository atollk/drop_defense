/**
 * Like [node_try_getter] but panics if the name or type don't match.
 */
#[macro_export]
macro_rules! node_getter {
    ($typ:ty, $name:literal) => {
        ::paste::paste! {
            $crate::node_getter_named!($typ, $name, [<get_node_ $name:snake>]);
        }
    };
}

/**
 * Like [node_getter] but specify a custom name for the getter.
 */
#[macro_export]
macro_rules! node_getter_named {
    ($typ:ty, $node_name:literal, $function_name:ident) => {
        fn $function_name(&self) -> godot::obj::Gd<$typ> {
            use godot::obj::WithBaseField;
            self.base().get_node_as($node_name)
        }
    };
}

/**
 * Creates a getter function (based on the given name) that retrieves the
 * child node named $name of type $typ.
 */
#[macro_export]
macro_rules! node_try_getter {
    ($typ:ty, $name:literal) => {
        ::paste::paste! {
            $crate::node_try_getter_named!($typ, $name, [<try_get_node_ $name:snake>]);
        }
    };
}

/**
 * Like [node_try_getter] but specify a custom name for the getter.
 */
#[macro_export]
macro_rules! node_try_getter_named {
    ($typ:ty, $node_name:literal, $function_name:ident) => {
        fn $function_name(&self) -> Option<godot::obj::Gd<$typ>> {
            use godot::obj::WithBaseField;
            self.base().try_get_node_as($node_name)
        }
    };
}
