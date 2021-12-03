use crate::vga::WRITER;
use crate::{serial_print, serial_println};
use alloc::{boxed::Box, format};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};
use futures_util::Stream;
use futures_util::StreamExt;
use futures_util::{task::AtomicWaker, Future};
use spin::Mutex;
use x86_64::instructions::{interrupts::without_interrupts, port::Port};

pub static READY: AtomicBool = AtomicBool::new(false);
static WAKER: AtomicWaker = AtomicWaker::new();

pub fn tick() {
    unsafe {
        let mut port0x70 = Port::new(0x70);
        let mut port0x71 = Port::new(0x71);
        port0x70.write(0x8Au8);
        let update: u8 = port0x71.read();
        if (update & (1 << 6)) == 0 {
            READY.swap(true, Ordering::Relaxed);
            WAKER.wake();
        }
    }
}

struct Timer;

impl Stream for Timer {
    type Item = (u8, u8, u8);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut port0x70 = Port::new(0x70);
        let mut port0x71 = Port::new(0x71);
        if READY.load(Ordering::Relaxed) {
            READY.store(false, Ordering::Relaxed);

            let time = unsafe {
                port0x70.write(0x80u8);
                let sec: u8 = port0x71.read();
                let sec = (sec >> 4) + (sec & 0xF);

                port0x70.write(0x82);
                let min = port0x71.read();
                let min = (min >> 4) + (min & 0xF);

                port0x70.write(0x84);
                let hour = port0x71.read();
                let hour = (hour >> 4) + (hour & 0xF);
                (hour, min, sec)
            };

            return Poll::Ready(Some(time));
        }

        WAKER.register(cx.waker());
        return if READY.load(Ordering::Relaxed) {
            WAKER.take();

            let time = unsafe {
                port0x70.write(0x80);
                let sec = port0x71.read();
                let sec = (sec >> 4) + (sec & 0xF);

                port0x70.write(0x82);
                let min = port0x71.read();
                let min = (min >> 4) + (min & 0xF);

                port0x70.write(0x84);
                let hour = port0x71.read();
                let hour = (hour >> 4) + (hour & 0xF);
                (hour, min, sec)
            };

            READY.store(false, Ordering::Relaxed);

            Poll::Ready(Some(time))
        } else {
            Poll::Pending
        };
    }
}

pub async fn print_time() {
    let mut timer = Timer;
    loop {
        let time = timer.next().await.unwrap();
        without_interrupts(|| WRITER.lock().write_time(time))
    }
}
