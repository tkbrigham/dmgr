#[macro_export]
macro_rules! dmgr_err {
    ($fmt:expr) => ($crate::DmgrErr::new($fmt));
    ($fmt:expr, $($args:tt)*) => ($crate::DmgrErr::new(&format!($fmt, $($args)*)));
}

#[macro_export]
macro_rules! err {
    ($fmt:expr) => (Err(dmgr_err!($fmt)));
    ($fmt:expr, $($args:tt)*) => (Err(dmgr_err!($fmt, $($args)*)));
}

#[macro_export]
macro_rules! fail {
    ($fmt:expr) => (return err!($fmt));
    ($fmt:expr, $($args:tt)*) => (return err!($fmt, $($args)*));
}
