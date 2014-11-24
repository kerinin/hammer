package db

import (
	"testing"
	"math/rand"
	"math/big"
)

var mkv10 *MapKV = NewMapKV()
var mkv100 *MapKV = NewMapKV()
var mkv1000 *MapKV = NewMapKV()
var mkv10000 *MapKV = NewMapKV()
var mkv100000 *MapKV = NewMapKV()

func init() {
	for i := 0; i < 10; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		mkv10.Add(key_int, key_big)
	}
	for i := 0; i < 100; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		mkv100.Add(key_int, key_big)
	}
	for i := 0; i < 1000; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		mkv1000.Add(key_int, key_big)
	}
	for i := 0; i < 10000; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		mkv10000.Add(key_int, key_big)
	}
	for i := 0; i < 100000; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)
		mkv100000.Add(key_int, key_big)
	}
}

func BenchmarkMap10(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		mkv10.Add(key_int, key_big)
		mkv10.Get(key_big)
		mkv10.Remove(key_int, key_big)
	}
}

func BenchmarkMap100(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		mkv100.Add(key_int, key_big)
		mkv100.Get(key_big)
		mkv100.Remove(key_int, key_big)
	}
}

func BenchmarkMap1000(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		mkv1000.Add(key_int, key_big)
		mkv1000.Get(key_big)
		mkv1000.Remove(key_int, key_big)
	}
}

func BenchmarkMap10000(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		mkv10000.Add(key_int, key_big)
		mkv10000.Get(key_big)
		mkv10000.Remove(key_int, key_big)
	}
}

func BenchmarkMap100000(b *testing.B) {
	for i := 0; i < b.N; i++ {
		key_int := big.NewInt(rand.Int63())
		key_big := NewKey(key_int)

		mkv100000.Add(key_int, key_big)
		mkv100000.Get(key_big)
		mkv100000.Remove(key_int, key_big)
	}
}
