use rlua::{Error, Context};

#[macro_export]
macro_rules! lua_err {
    ($($t:tt)*) => {
        return Err(rlua::Error::RuntimeError(format!($($t)*)))
    };
}

pub trait LuaInit {
    fn initialize_lua(ctx: Context) -> Result<(), Error>;
}
