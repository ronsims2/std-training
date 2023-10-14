// Reference: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/freertos.html
use anyhow::Result;
use esp_idf_sys::{
    esp, gpio_config, gpio_config_t, gpio_install_isr_service,
    gpio_int_type_t_GPIO_INTR_POSEDGE, gpio_isr_handler_add, gpio_mode_t_GPIO_MODE_INPUT,
    xQueueGenericCreate, xQueueGiveFromISR, xQueueReceive, QueueHandle_t, ESP_INTR_FLAG_IRAM,
};


// These imports are needed for part 2.

use log::{info};

// 4. Create a `static mut` that holds the queue handle.
static mut EVENT_QUEUE: Option<QueueHandle_t> = None;

// 6. Define what the interrupt handler does, once the button is pushed. Button_interrupt sends a message into the queue.
#[link_section = ".iram0.text"]
unsafe extern "C" fn button_interrupt(_: *mut core::ffi::c_void) {
    xQueueGiveFromISR(EVENT_QUEUE.unwrap(), std::ptr::null_mut());
}

fn main() -> Result<()> {
    const GPIO_NUM: i32 = 9;

    // 1. Add GPIO configuration C struct
    // let io_conf = gpio_config_t {
    //     ...
    // };
    let io_conf = gpio_config_t {
        pin_bit_mask: 1 << GPIO_NUM, // Have no idea why its shifted this way
        mode: gpio_mode_t_GPIO_MODE_INPUT,
        pull_up_en: true.into(),
        pull_down_en: false.into(),
        intr_type: gpio_int_type_t_GPIO_INTR_POSEDGE
    };

    unsafe {
        // 2. Write the GPIO configuration into the register
        // esp!(...)?;
        esp!(gpio_config(&io_conf))?;

        // 3. Install the global GPIO interrupt handler
        // esp!(...)?;
        esp!(gpio_install_isr_service(ESP_INTR_FLAG_IRAM.try_into().unwrap()))?;

        // Queue configurations
        const QUEUE_TYPE_BASE: u8 = 0;
        const ITEM_SIZE: u32 = 0;
        const QUEUE_SIZE: u32 = 1;

        // 5. Create an event queue
        // EVENT_QUEUE = Some(...);
        EVENT_QUEUE = Some(xQueueGenericCreate(QUEUE_SIZE, ITEM_SIZE, QUEUE_TYPE_BASE));

        // 7. Add the button GPIO and the function to the interrupt handler
        // esp!(...)?;
        esp!(gpio_isr_handler_add(GPIO_NUM, Some(button_interrupt), std::ptr::null_mut()))?;
    }

    // The loop in main waits until it gets a message through the rx ("receiver") part of the channel
    loop {
        unsafe {
            // Maximum delay
            const QUEUE_WAIT_TICKS: u32 = 1000;;

            // 8. Receive the event from the queue.
            // let res = ...;
            let res = xQueueReceive(EVENT_QUEUE.unwrap(), std::ptr::null_mut(), QUEUE_WAIT_TICKS);
            match res {
                1 => info!("The button has been pressed, result: {}", res),
                _ => {}
            };


            // 9. Handle the value of res.
            // ...
        }
    }
}
