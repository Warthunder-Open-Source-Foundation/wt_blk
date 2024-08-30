use color_eyre::eyre::eyre;
use color_eyre::Report;

pub type BlkError = &'static str;


pub trait IexToEyre {
	type Element;
	fn into_eyre<Element>(self) -> Result<Element, Report>;
}
impl <T>IexToEyre for Result<T, &'static str> {
	type Element = T;

	fn into_eyre<Element>(self) -> Result<Element, Report> {
		self.map_err(|e|eyre!("{e}"))
	}
}