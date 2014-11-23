package db

type KV interface {
	Get(key interface{}) ([]Key, bool)
	Add(key interface{}, value Key) bool
	Remove(key interface{}, value Key) bool
}
