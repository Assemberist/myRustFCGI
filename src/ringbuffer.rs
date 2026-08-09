use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::limits;

pub struct SpscRingBuffer<Type, const LENGTH: usize> {
    data: [Cell<Type>; LENGTH],
    read_index: AtomicUsize,
    write_index: AtomicUsize,
}

unsafe impl<Type: Send, const LENGTH: usize> Sync for SpscRingBuffer<Type, LENGTH> {}

impl<Type: Default, const LENGTH: usize> SpscRingBuffer<Type, LENGTH> {
    pub fn new() -> Self {
        let constructed_data = std::array::from_fn(|_| Cell::new(Type::default()));
        
        Self {
            data: constructed_data,
            read_index: AtomicUsize::new(0),
            write_index: AtomicUsize::new(0),
        }
    }

    /// Добавляет элемент в буфер. Возвращает true в случае успеха.
    /// Вызывается строго из ОДНОГО потока (Producer).
    pub fn put(&self, item: Type) -> bool {
        let write = self.write_index.load(Ordering::Relaxed);
        // Читаем read_index с Acquire, чтобы синхронизировать память с Consumer'ом
        let read = self.read_index.load(Ordering::Acquire);

        // Буфер полон, если следующий индекс записи догонит индекс чтения
        if (write + 1) % limits::MAX_CONN == read % limits::MAX_CONN {
            return false; 
        }

        let target_cell = &self.data[write % limits::MAX_CONN];
        
        // Безопасно заменяем дефолтное значение на наше новое
        target_cell.set(item);

        // Публикуем новый индекс записи. Release гарантирует, что запись
        // в ячейку выше завершится ДО того, как обновится индекс.
        self.write_index.store(write + 1, Ordering::Release);
        true
    }

    /// Забирает элемент из буфера.
    /// Вызывается строго из ОДНОГО потока (Consumer).
    pub fn take(&self) -> Option<Type> {
        let read = self.read_index.load(Ordering::Relaxed);
        // Читаем write_index с Acquire, чтобы увидеть актуальные данные, записанные Producer'ом
        let write = self.write_index.load(Ordering::Acquire);

        // Буфер пуст
        if read == write {
            return None;
        }

        let target_cell = &self.data[read % limits::MAX_CONN];
        
        // Забираем значение из ячейки, оставляя на его месте дефолтное (Type::default())
        let item = target_cell.take();

        // Публикуем новый индекс чтения. Release сигнализирует Producer'у,
        // что ячейка освободилась (в ней снова лежит дефолтное значение).
        self.read_index.store(read + 1, Ordering::Release);
        Some(item)
    }
}