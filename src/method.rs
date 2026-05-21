use std::{
    fmt::{Debug, Display},
    net::IpAddr,
};

use crate::{allowlist::AllowList, dataset::Datum};

pub mod bfcms;
pub mod ibhh;
pub mod uniqd;

pub use bfcms::BfcmsMethod;
use ibhh::IbhhAlertSummary;
pub use ibhh::IbhhMethod;
pub use uniqd::UniqdMethod;

pub enum IbhhMethodClass<L: AllowList> {
    Ibhh(IbhhMethod<L>),
}

impl<L: AllowList + 'static> DetectionMethod for IbhhMethodClass<L> {
    type TAlert = IbhhAlertSummary;

    fn reset_interval(&self) -> u32 {
        match self {
            IbhhMethodClass::Ibhh(ibhh_method) => ibhh_method.reset_interval(),
        }
    }

    fn process_single(
        &mut self,
        datum: Datum<'_>,
    ) -> color_eyre::Result<(Option<Self::TAlert>, f64)> {
        match self {
            IbhhMethodClass::Ibhh(ibhh_method) => ibhh_method.process_single(datum),
        }
    }
}

#[allow(unused)]
pub trait AlertSummary: Debug + Display {
    type AlertKind: Debug + Display;

    fn kind(&self) -> Self::AlertKind;
    fn domains(&self) -> impl Iterator<Item = (&str, f64)>;
    fn clients(&self) -> impl Iterator<Item = (IpAddr, u32)>;
}

pub trait DetectionMethod {
    type TAlert: AlertSummary;

    /// The reset interval of the method in milliseconds
    fn reset_interval(&self) -> u32;

    fn process_single(
        &mut self,
        datum: Datum<'_>,
    ) -> color_eyre::Result<(Option<Self::TAlert>, f64)>;
}
