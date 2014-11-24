package db

import (
	"sync"
)

type MapKV struct {
	kv map[interface{}][]Key
	mutex sync.RWMutex
}


func NewMapKV() *MapKV {
	kv := make(map[interface{}][]Key)
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

func (l *MapKV) Get(key interface{}) ([]Key, bool) {
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
		for _, found_value := range found_values {
			if found_value.Cmp(value) == 0 {
				l.mutex.Unlock()
				return false
			}
		}
		l.kv[key] = append(found_values, value)
		l.mutex.Unlock()
		return true

	} else {
		l.kv[key] = []Key{value}
		l.mutex.Unlock()
		return true
	}
}

func (l *MapKV) Remove(key interface{}, value Key) bool {
	l.mutex.Lock()
	found_values, ok := l.kv[key]

	if ok {
		if len(found_values) == 1 {
			delete(l.kv, key)
			l.mutex.Unlock()
			return true

		} else {
			for i, found_value := range found_values {
				if found_value.Cmp(value) == 0 {
					// Seriously, THIS is how I have to delete elements in Go?!?!?!
					copy(found_values[i:], found_values[i+1:])
					l.kv[key] = found_values[:len(found_values)-1]

					l.mutex.Unlock()
					return true
				}
			}
		}
	}

	l.mutex.Unlock()
	return false
}
