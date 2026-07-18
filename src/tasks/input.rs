//! embassy task: Input
//! - polls GPIO buttons
//! - polls I2C sensors (via multiplexer)
//! - debounces buttons
//! - any processing on the axes
//! - broadcasts HardwareState
