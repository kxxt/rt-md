use serde::{Deserialize, Serialize};

use crate::dataset::ours::OurDatasetConfig;

use super::ziza::ZizaDatasetConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DatasetConfig {
    Ziza(ZizaDatasetConfig),
    Ours(OurDatasetConfig),
}
