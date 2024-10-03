use frame::prelude::*;

#[frame::barrel(dev_mode)]
pub mod barrel {
	use super::*;

	#[barrel::config]
	pub trait Config: frame_system::Config {}

	#[barrel::barrel]
	pub struct Barrel<T>(_);
}
