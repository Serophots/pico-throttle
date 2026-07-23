pub trait ResultExt {
    type Ok;

    fn err_log(self) -> Option<Self::Ok>;
}

impl<T, E: core::fmt::Debug + defmt::Format> ResultExt for Result<T, E> {
    type Ok = T;

    #[track_caller]
    fn err_log(self) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(err) => {
                defmt::error!("{:?}", err);
                None
            }
        }
    }
}
