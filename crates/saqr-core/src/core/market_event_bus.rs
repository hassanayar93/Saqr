use parking_lot::Mutex;

use super::lock_free_ring_buffer::LockFreeRingBuffer;
use super::market_event::MarketEvent;

/// جسر SPSC آمن بين مهمة WebSocket ومهمة hot_loop (قفل قصير — لا deadlocks متعددة).
pub struct MarketEventBus {
    inner: Mutex<LockFreeRingBuffer>,
}

impl MarketEventBus {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(LockFreeRingBuffer::new()),
        }
    }

    #[inline]
    pub fn try_push(&self, event: MarketEvent) -> Result<(), &'static str> {
        self.inner.lock().try_push(event)
    }

    #[inline]
    pub fn try_pop(&self) -> Option<MarketEvent> {
        self.inner.lock().try_pop()
    }

    pub fn pending_estimate(&self) -> u64 {
        let guard = self.inner.lock();
        guard
            .tail()
            .saturating_sub(guard.head())
    }
}

impl Default for MarketEventBus {
    fn default() -> Self {
        Self::new()
    }
}
