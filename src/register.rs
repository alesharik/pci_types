use bit_field::BitField;
use core::convert::TryFrom;
use core::fmt::{Debug, Formatter};

/// Slowest time that a device will assert DEVSEL# for any bus command except Configuration Space
/// read and writes
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DevselTiming {
    Fast = 0x0,
    Medium = 0x1,
    Slow = 0x2,
}

impl TryFrom<u8> for DevselTiming {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(DevselTiming::Fast),
            0x1 => Ok(DevselTiming::Medium),
            0x2 => Ok(DevselTiming::Slow),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct StatusRegister(u16);

impl StatusRegister {
    pub fn new(value: u16) -> Self {
        StatusRegister(value)
    }

    /// Will be `true` whenever the device detects a parity error, even if parity error handling is disabled.
    pub fn parity_error_detected(&self) -> bool {
        self.0.get_bit(15)
    }

    /// Will be `true` whenever the device asserts SERR#.
    pub fn signalled_system_error(&self) -> bool {
        self.0.get_bit(14)
    }

    /// Will return `true`, by a master device, whenever its transaction
    /// (except for Special Cycle transactions) is terminated with Master-Abort.
    pub fn received_master_abort(&self) -> bool {
        self.0.get_bit(13)
    }

    /// Will return `true`, by a master device, whenever its transaction is terminated with Target-Abort.
    pub fn received_target_abort(&self) -> bool {
        self.0.get_bit(12)
    }

    /// Will return `true` whenever a target device terminates a transaction with Target-Abort.
    pub fn signalled_target_abort(&self) -> bool {
        self.0.get_bit(11)
    }

    /// The slowest time that a device will assert DEVSEL# for any bus command except
    /// Configuration Space read and writes.
    ///
    /// For PCIe always set to `Fast`
    pub fn devsel_timing(&self) -> Result<DevselTiming, ()> {
        let bits = self.0.get_bits(9..11);
        DevselTiming::try_from(bits as u8)
    }

    /// This returns `true` only when the following conditions are met:
    /// - The bus agent asserted PERR# on a read or observed an assertion of PERR# on a write
    /// - the agent setting the bit acted as the bus master for the operation in which the error occurred
    /// - bit 6 of the Command register (Parity Error Response bit) is set to 1.
    pub fn master_data_parity_error(&self) -> bool {
        self.0.get_bit(8)
    }

    /// If returns `true` the device can accept fast back-to-back transactions that are not from
    /// the same agent; otherwise, transactions can only be accepted from the same agent.
    ///
    /// For PCIe always set to `false`
    pub fn fast_back_to_back_capable(&self) -> bool {
        self.0.get_bit(7)
    }

    /// If returns `true` the device is capable of running at 66 MHz; otherwise, the device runs at 33 MHz.
    ///
    /// For PCIe always set to `false`
    pub fn capable_66mhz(&self) -> bool {
        self.0.get_bit(5)
    }

    /// If returns `true` the device implements the pointer for a New Capabilities Linked list;
    /// otherwise, the linked list is not available.
    ///
    /// For PCIe always set to `true`
    pub fn has_capability_list(&self) -> bool {
        self.0.get_bit(4)
    }

    /// Represents the state of the device's INTx# signal. If returns `true` and bit 10 of the
    /// Command register (Interrupt Disable bit) is set to 0 the signal will be asserted;
    /// otherwise, the signal will be ignored.
    pub fn interrupt_status(&self) -> bool {
        self.0.get_bit(3)
    }
}

impl Debug for StatusRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StatusRegister")
            .field("parity_error_detected", &self.parity_error_detected())
            .field("signalled_system_error", &self.signalled_system_error())
            .field("received_master_abort", &self.received_master_abort())
            .field("received_target_abort", &self.received_target_abort())
            .field("signalled_target_abort", &self.signalled_target_abort())
            .field("devsel_timing", &self.devsel_timing())
            .field("master_data_parity_error", &self.master_data_parity_error())
            .field(
                "fast_back_to_back_capable",
                &self.fast_back_to_back_capable(),
            )
            .field("capable_66mhz", &self.capable_66mhz())
            .field("has_capability_list", &self.has_capability_list())
            .field("interrupt_status", &self.interrupt_status())
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct CommandRegister(u16);

impl CommandRegister {
    pub(crate) fn from_u16(data: u16) -> CommandRegister {
        CommandRegister(data)
    }

    pub(crate) fn write_info(&self, data: &mut u32) {
        data.set_bits(8..=10, self.0.get_bits(8..=10) as u32);
        data.set_bits(0..=6, self.0.get_bits(0..=6) as u32);
    }

    pub fn builder(&self) -> CommandRegisterBuilder {
        CommandRegisterBuilder(self.0)
    }

    /// If `true` the assertion of the devices INTx# signal is disabled;
    /// otherwise, assertion of the signal is enabled.
    pub fn interrupts_disabled(&self) -> bool {
        self.0.get_bit(10)
    }

    /// If `true` indicates a device is allowed to generate fast back-to-back transactions;
    /// otherwise, fast back-to-back transactions are only allowed to the same agent.
    pub fn fast_back_to_back_enabled(&self) -> bool {
        self.0.get_bit(9)
    }

    /// If `true` the SERR# driver is enabled;
    /// otherwise, the driver is disabled.
    pub fn serr_enabled(&self) -> bool {
        self.0.get_bit(8)
    }

    /// If `true` the device will take its normal action when a parity error is detected;
    /// otherwise, when an error is detected, the device will set bit 15 of the Status register
    /// (Detected Parity Error Status Bit), but will not assert the PERR# (Parity Error)
    /// pin and will continue operation as normal.
    pub fn parity_error_response(&self) -> bool {
        self.0.get_bit(6)
    }

    /// If `true` the device does not respond to palette register writes and will snoop the data;
    /// otherwise, the device will treat palette write accesses like all other accesses.
    pub fn vga_palette_snoop(&self) -> bool {
        self.0.get_bit(5)
    }

    /// If `true` the device can generate the Memory Write and Invalidate command;
    /// otherwise, the Memory Write command must be used.
    pub fn memory_write_and_invalidate_enabled(&self) -> bool {
        self.0.get_bit(4)
    }

    /// If `true` the device can monitor Special Cycle operations;
    /// otherwise, the device will ignore them.
    pub fn monitor_special_cycles(&self) -> bool {
        self.0.get_bit(3)
    }

    /// If `true` the device can behave as a bus master;
    /// otherwise, the device can not generate PCI accesses.
    pub fn bus_mastering_enabled(&self) -> bool {
        self.0.get_bit(2)
    }

    /// If `true` the device can respond to Memory Space accesses;
    /// otherwise, the device's response is disabled.
    pub fn memory_space_access_enabled(&self) -> bool {
        self.0.get_bit(1)
    }

    /// If `true` the device can respond to I/O Space accesses;
    /// otherwise, the device's response is disabled.
    pub fn io_space_access_enabled(&self) -> bool {
        self.0.get_bit(0)
    }
}

impl Debug for CommandRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CommandRegister")
            .field("interrupts_disabled", &self.interrupts_disabled())
            .field(
                "fast_back_to_back_enabled",
                &self.fast_back_to_back_enabled(),
            )
            .field("serr_enabled", &self.serr_enabled())
            .field("parity_error_response", &self.parity_error_response())
            .field("vga_palette_snoop", &self.vga_palette_snoop())
            .field(
                "memory_write_and_invalidate_enabled",
                &self.memory_write_and_invalidate_enabled(),
            )
            .field("monitor_special_cycles", &self.monitor_special_cycles())
            .field("bus_mastering_enabled", &self.bus_mastering_enabled())
            .field(
                "memory_space_access_enabled",
                &self.memory_space_access_enabled(),
            )
            .field("io_space_access_enabled", &self.io_space_access_enabled())
            .finish()
    }
}

#[derive(Default)]
pub struct CommandRegisterBuilder(u16);

impl CommandRegisterBuilder {
    /// If `true` the assertion of the devices INTx# signal is disabled;
    /// otherwise, assertion of the signal is enabled.
    pub fn interrupt_disable(&mut self, disable: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(10, disable);
        self
    }

    /// If `true` indicates a device is allowed to generate fast back-to-back transactions;
    /// otherwise, fast back-to-back transactions are only allowed to the same agent.
    pub fn fast_back_to_back_enable(&mut self, enable: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(9, enable);
        self
    }

    /// If `true` the SERR# driver is enabled;
    /// otherwise, the driver is disabled.
    pub fn serr_enable(&mut self, enable: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(8, enable);
        self
    }

    /// If `true` the device will take its normal action when a parity error is detected;
    /// otherwise, when an error is detected, the device will set bit 15 of the Status register
    /// (Detected Parity Error Status Bit), but will not assert the PERR# (Parity Error)
    /// pin and will continue operation as normal.
    pub fn parity_error_response(&mut self, response: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(6, response);
        self
    }

    /// If `true` the device does not respond to palette register writes and will snoop the data;
    /// otherwise, the device will treat palette write accesses like all other accesses.
    pub fn vga_palette_snoop(&mut self, snoop: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(5, snoop);
        self
    }

    /// If `true` the device can generate the Memory Write and Invalidate command;
    /// otherwise, the Memory Write command must be used.
    pub fn memory_write_and_invalidate_enable(
        &mut self,
        enable: bool,
    ) -> &mut CommandRegisterBuilder {
        self.0.set_bit(4, enable);
        self
    }

    /// If `true` the device can monitor Special Cycle operations;
    /// otherwise, the device will ignore them.
    pub fn monitor_special_cycles(&mut self, cycles: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(3, cycles);
        self
    }

    /// If `true` the device can behave as a bus master;
    /// otherwise, the device can not generate PCI accesses.
    pub fn bus_mastering(&mut self, mastering: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(2, mastering);
        self
    }

    /// If `true` the device can respond to Memory Space accesses;
    /// otherwise, the device's response is disabled.
    pub fn memory_space_access(&mut self, access: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(1, access);
        self
    }

    /// If `true` the device can respond to I/O Space accesses;
    /// otherwise, the device's response is disabled.
    pub fn io_space_access(&mut self, access: bool) -> &mut CommandRegisterBuilder {
        self.0.set_bit(0, access);
        self
    }

    pub fn build(self) -> CommandRegister {
        CommandRegister(self.0)
    }
}
