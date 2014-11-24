package db

import (
	"github.com/hashicorp/golang-lru"
)

type LruKV struct {
	kv    *lru.Cache
}

func NewLruKV(lru_size int) *LruKV {
	kv, _ := lru.New(lru_size)

	return &LruKV{kv: kv}
}

func NewLruPartitioning(bits, tolerance uint, lru_size int) Partitioning {
	return NewPartitioning(bits, tolerance, func(shift, mask uint) Partition {
		return Partition{
			shift: shift,
			mask: mask,
			zero_kv: KV(NewLruKV(lru_size)),
			one_kv: KV(NewLruKV(lru_size)),
		}
	})
}

func (l *LruKV) Get(key interface{}) ([]Key, bool) {
	found_values, ok := l.kv.Get(key)

	if ok {
		return found_values.([]Key), true
	} else {
		return make([]Key, 0, 0), false
	}
}

func (l *LruKV) Add(key interface{}, value Key) bool {
	found_values, ok := l.kv.Get(key)

	if ok {
		for _, found_value := range found_values.([]Key) {
			if found_value.Cmp(value) == 0 {
				return false
			}
		}
		l.kv.Add(key, append(found_values.([]Key), value))

	} else {
		l.kv.Add(key, []Key{value})
	}

	return true
}

func (l *LruKV) Remove(key interface{}, value Key) bool {
	found_values, ok := l.kv.Get(key)

	if ok {
		if len(found_values.([]Key)) == 1 {
			l.kv.Remove(key)
			return true

		} else {
			for i, found_value := range found_values.([]Key) {
				if found_value.Cmp(value) == 0 {
					// Seriously, THIS is how I have to delete elements in Go?!?!?!
					copy(found_values.([]Key)[i:], found_values.([]Key)[i+1:])
					l.kv.Add(key, found_values.([]Key)[:len(found_values.([]Key))-1])

					return true
				}
			}
		}
	}

	return false
}
