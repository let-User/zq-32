// в main.rs или в отдельном модуле

// Глобальная переменная для счётчика
static mut TICKS_UNTIL_REBOOT: u64 = 6000; // 1 минута при 100 Гц

// В главном цикле, после обработки ввода:
unsafe {
    if TICKS_UNTIL_REBOOT > 0 {
        TICKS_UNTIL_REBOOT -= 1;
    } else {
        acpi_reboot(); // Эта функция должна быть в модуле acpi.rs
    }
}
