use std::ops::{Add, Div, RangeInclusive, Sub};

/**
 * Like [node_try_getter] but panics if the name or type don't match.
 */
#[macro_export]
macro_rules! node_getter {
    // Entry point
    ($first:ident $(/ $rest:ident)* : $typ:ty) => {
        $crate::node_getter!(
            @last
            [concat!(stringify!($first) $(, "/", stringify!($rest))*)]
            $first $($rest)* : $typ
        );
    };

    // Exactly one ident left: it's the last one, so emit
    (@last [$path:expr] $last:ident : $typ:ty) => {
        ::paste::paste! {
            $crate::node_getter_named!($typ, $path, [<get_node_ $last:snake>]);
        }
    };

    // More than one left: drop the first and recurse
    (@last [$path:expr] $x:ident $($rest:ident)+ : $typ:ty) => {
        $crate::node_getter!(@last [$path] $($rest)+ : $typ);
    };
}

/**
 * Like [node_getter] but specify a custom name for the getter.
 */
#[macro_export]
macro_rules! node_getter_named {
    ($typ:ty, $node_name:expr, $vis:vis $function_name:ident) => {
        $vis fn $function_name(&self) -> godot::obj::Gd<$typ> {
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
    // Entry point
    ($first:ident $(/ $rest:ident)* : $typ:ty) => {
        $crate::node_try_getter!(
            @last
            [concat!(stringify!($first) $(, "/", stringify!($rest))*)]
            $first $($rest)* : $typ
        );
    };

    // Exactly one ident left: it's the last one, so emit
    (@last [$path:expr] $last:ident : $typ:ty) => {
        ::paste::paste! {
            $crate::node_try_getter_named!($typ, $path, [<get_node_ $last:snake>]);
        }
    };

    // More than one left: drop the first and recurse
    (@last [$path:expr] $x:ident $($rest:ident)+ : $typ:ty) => {
        $crate::node_try_getter!(@last [$path] $($rest)+ : $typ);
    };
}

/**
 * Like [node_try_getter] but specify a custom name for the getter.
 */
#[macro_export]
macro_rules! node_try_getter_named {
    ($typ:ty, $node_name:literal, $vis:vis $function_name:ident) => {
        $vis fn $function_name(&self) -> Option<godot::obj::Gd<$typ>> {
            use godot::obj::WithBaseField;
            self.base().try_get_node_as($node_name)
        }
    };
}

/// Creates a range from a given center point and the total range width.
pub fn range_from_center<T>(center: T, width: T) -> RangeInclusive<T>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + Div<Output = T> + From<u8>,
{
    let half = width / T::from(2u8);
    (center - half)..=(center + half)
}
