package db

import (
	"testing"
	"math/rand"
	"math/big"
)

var lkv10 *LruKV = NewLruKV(10)
var lkv100 *LruKV = NewLruKV(100)
var lkv1000 *LruKV = NewLruKV(1000)
var lkv10000 *LruKV = NewLruKV(10000)
var lkv100000 *LruKV = NewLruKV(100000)

func init() {
	for i := 0; i < 10; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		lkv10.Add(key_int, key_big)
	}
	for i := 0; i < 100; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		lkv100.Add(key_int, key_big)
	}
	for i := 0; i < 1000; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		lkv1000.Add(key_int, key_big)
	}
	for i := 0; i < 10000; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		lkv10000.Add(key_int, key_big)
	}
	for i := 0; i < 100000; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		lkv100000.Add(key_int, key_big)
	}
}

func BenchmarkLRU10(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		lkv10.Add(key_int, key_big)
		lkv10.Get(key_big)
		lkv10.Remove(key_int, key_big)
	}
}

func BenchmarkLRU100(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		lkv100.Add(key_int, key_big)
		lkv100.Get(key_big)
		lkv100.Remove(key_int, key_big)
	}
}

func BenchmarkLRU1000(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		lkv1000.Add(key_int, key_big)
		lkv1000.Get(key_big)
		lkv1000.Remove(key_int, key_big)
	}
}

func BenchmarkLRU10000(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		lkv10000.Add(key_int, key_big)
		lkv10000.Get(key_big)
		lkv10000.Remove(key_int, key_big)
	}
}

func BenchmarkLRU100000(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		lkv100000.Add(key_int, key_big)
		lkv100000.Get(key_big)
		lkv100000.Remove(key_int, key_big)
	}
}
