package db

import (
	"sync"
)

type MapKV struct {
	kv map[interface{}]*map[Key]bool
	mutex sync.RWMutex
}


func NewMapKV() *MapKV {
	kv := make(map[interface{}]*map[Key]bool)
	mutex := sync.RWMutex{}

	return &MapKV{kv: kv, mutex: mutex}
}

func NewMapPartitioning(bits, tolerance uint) Partitioning {
	return NewPartitioning(bits, tolerance, func(shift, mask uint) Partition {
		return Partition{
			shift: shift,
			mask: mask,
			zero_kv: KV(NewMapKV()),
			one_kv: KV(NewMapKV()),
		}
	})
}

func (l *MapKV) Get(key interface{}) (*map[Key]bool, bool) {
	l.mutex.RLock()
	source_keys, ok := l.kv[key]
	l.mutex.RUnlock()

	return source_keys, ok
}

func (l *MapKV) Add(key interface{}, value Key) bool {
	l.mutex.Lock()
	found_values, ok := l.kv[key]

	// `key` exists (multiple values can have the same key)
	if ok {
		_, ok := (*found_values)[value]
		// `value` exists
		if ok {
			l.mutex.Unlock()
			return false
		} else {
			(*found_values)[value] = true
			l.mutex.Unlock()
			return true
		}
	} else {
		l.kv[key] = &map[Key]bool{value: true}
		l.mutex.Unlock()
		return true
	}
}

func (l *MapKV) Remove(key interface{}, value Key) bool {
	l.mutex.Lock()
	found_values, ok := l.kv[key]

	if ok {
		_, ok := (*found_values)[value]
		if ok {
			delete(*found_values, value)
			if len(*found_values) == 0 {
				delete(l.kv, key)
			}
			l.mutex.Unlock()
			return true
		} else {
			l.mutex.Unlock()
			return false
		}
	} else {
		l.mutex.Unlock()
		return false
	}
}
