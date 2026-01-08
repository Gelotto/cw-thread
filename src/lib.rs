#[cfg(not(feature = "library"))]
pub mod contract;
pub mod error;
#[cfg(not(feature = "library"))]
pub mod execute;
pub mod msg;
#[cfg(not(feature = "library"))]
pub mod query;
pub mod state;
mod util;
mod validation;

#[cfg(test)]
mod tests;
