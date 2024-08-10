pub mod agaudio;

#[cfg_attr(feature = "pcbv1", path = "agdisplay_st7735.rs")]
#[cfg_attr(not(feature = "pcbv1"), path = "agdisplay_st7789.rs")]
pub mod agdisplay;

pub mod aginput;
pub mod peripherals_cfg;
