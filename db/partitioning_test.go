package db

import (
	// "fmt"
	"math/big"
	"math/rand"
	"reflect"
	"testing"
)

func TestPartitioningPartitionEvenly(t *testing.T) {
	partitioning := NewLruPartitioning(32, 5, 10)
	expected_partitions := make([]Partition, 4, 4)

	expected_partitions[0] = Partition{shift: 0, mask: 8, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[1] = Partition{shift: 8, mask: 8, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[2] = Partition{shift: 16, mask: 8, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[3] = Partition{shift: 24, mask: 8, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningPartitionUnevenly(t *testing.T) {
	partitioning := NewLruPartitioning(32, 7, 10)
	expected_partitions := make([]Partition, 5, 5)

	expected_partitions[0] = Partition{shift: 0, mask: 7, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[1] = Partition{shift: 7, mask: 7, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[2] = Partition{shift: 14, mask: 6, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[3] = Partition{shift: 20, mask: 6, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[4] = Partition{shift: 26, mask: 6, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningPartitionTooMany(t *testing.T) {
	partitioning := NewLruPartitioning(4, 8, 10)
	expected_partitions := make([]Partition, 3, 3)

	expected_partitions[0] = Partition{shift: 0, mask: 2, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[1] = Partition{shift: 2, mask: 1, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}
	expected_partitions[2] = Partition{shift: 3, mask: 1, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningPartitionZero(t *testing.T) {
	partitioning := NewLruPartitioning(32, 0, 10)
	expected_partitions := make([]Partition, 1, 1)

	expected_partitions[0] = Partition{shift: 0, mask: 32, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningPartitionWithNoBytes(t *testing.T) {
	partitioning := NewLruPartitioning(0, 0, 10)
	expected_partitions := make([]Partition, 1, 1)

	expected_partitions[0] = Partition{shift: 0, mask: 0, zero_kv: NewLruKV(10), one_kv: NewLruKV(10)}

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningFindMissingKey(t *testing.T) {
	partitioning := NewLruPartitioning(8, 2, 10)
	a := NewKeyFromBinaryString("11111111")
	keys, err := partitioning.Find(a)

	if len(keys) != 0 {
		t.Logf("Find returned non-empty set: %v", keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningInsertFirstKey(t *testing.T) {
	partitioning := NewLruPartitioning(8, 2, 10)
	a := NewKeyFromBinaryString("11111111")

	inserted, err := partitioning.Insert(a)
	if !inserted {
		t.Logf("Insert returned false")
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningInsertSecondKey(t *testing.T) {
	partitioning := NewLruPartitioning(4, 4, 10)
	a := NewKeyFromBinaryString("00001111")
	b := NewKeyFromBinaryString("00001111")

	inserted, err := partitioning.Insert(a)
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}

	inserted, err = partitioning.Insert(b)
	if inserted {
		t.Logf("Insert returned true")
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningFindInsertedKey(t *testing.T) {
	partitioning := NewLruPartitioning(4, 4, 10)
	a := NewKeyFromBinaryString("00001111")
	b := NewKeyFromBinaryString("00001111")
	expected := map[Key]uint{a: 0}

	_, err := partitioning.Insert(a)
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}

	keys, err := partitioning.Find(b)
	if !reflect.DeepEqual(keys, expected) {
		t.Logf("Find returned unexpected set (expected %v): %v", expected, keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningFindPermutationOfInsertedKey(t *testing.T) {
	partitioning := NewLruPartitioning(4, 4, 10)
	a := NewKeyFromBinaryString("00001111")
	b := NewKeyFromBinaryString("00000111")
	expected := map[Key]uint{a: 1}

	partitioning.Insert(a)
	keys, err := partitioning.Find(b)
	if !reflect.DeepEqual(keys, expected) {
		t.Logf("Find returned unexpected set (expected %v): %v", expected, keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningFindPermutationsOfMultipleSimilarKeys(t *testing.T) {
	partitioning := NewLruPartitioning(8, 4, 10)
	a := NewKeyFromBinaryString("00000000")
	b := NewKeyFromBinaryString("10000000")
	c := NewKeyFromBinaryString("10000001")
	d := NewKeyFromBinaryString("11000001")
	e := NewKeyFromBinaryString("11000011")

	expected := map[Key]uint{b: 1, c: 2, d: 3, e: 4}

	partitioning.Insert(b)
	partitioning.Insert(c)
	partitioning.Insert(d)
	partitioning.Insert(e)

	keys, err := partitioning.Find(a)
	if !reflect.DeepEqual(keys, expected) {
		t.Logf("Find returned unexpected set (expected %v): %v", expected, keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningDontFindPermutationOfInsertedKey(t *testing.T) {
	partitioning := NewLruPartitioning(4, 2, 10)
	a := NewKeyFromBinaryString("00001111")
	b := NewKeyFromBinaryString("00110011")

	partitioning.Insert(a)
	keys, err := partitioning.Find(b)
	if len(keys) != 0 {
		t.Logf("Find returned non-empty set: %v", keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningRemoveInsertedKey(t *testing.T) {
	partitioning := NewLruPartitioning(4, 4, 10)
	a := NewKeyFromBinaryString("00001111")
	b := NewKeyFromBinaryString("00001111")
	c := NewKeyFromBinaryString("00001111")

	partitioning.Insert(a)

	removed, err := partitioning.Remove(b)
	if !removed {
		t.Logf("Remove returned false")
		t.Fail()
	}
	if err != nil {
		t.Logf("Remove returned error: %v", err)
		t.Fail()
	}

	keys, err := partitioning.Find(c)
	if len(keys) != 0 {
		t.Logf("Find returned non-empty set: %v", keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningRemoveMissingKey(t *testing.T) {
	partitioning := NewLruPartitioning(4, 4, 10)
	a := NewKeyFromBinaryString("00001111")

	removed, err := partitioning.Remove(a)
	if removed {
		t.Logf("Remove returned true")
		t.Fail()
	}
	if err != nil {
		t.Logf("Remove returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningConsistencyUnderLoad(t *testing.T) {
	partitioning := NewLruPartitioning(16, 4, 100000)

	var expected_present [65536]bool
	var expected_absent [65536]bool

	for i := 0; i < 100000; i++ {
		j := rand.Uint32() % 16
		j_big := NewKey(big.NewInt(int64(j)))

		if expected_present[j] {
			_, err := partitioning.Remove(j_big)
			if err != nil {
				t.Error(err)
			}

			expected_present[j] = false
			expected_absent[j] = true
		} else {
			_, err := partitioning.Insert(j_big)
			if err != nil {
				t.Error(err)
			}

			expected_present[j] = true
			expected_absent[j] = false
		}

		// Every 1000 operations check for failure
		if i%1000 == 0 {
			for k, b := range expected_present {
				if b {
					k_big := NewKey(big.NewInt(int64(k)))
					keys, err := partitioning.Find(k_big)
					if err != nil {
						t.Error(err)
					}

					found := false
					for key, _ := range keys {
						if key.Cmp(k_big) == 0 {
							found = true
						}
					}
					if !found {
						t.Logf("Expected to find %v in %v", k_big, keys)
						t.Fail()
						return
					}
				}
			}
			for k, b := range expected_absent {
				if b {
					k_big := NewKey(big.NewInt(int64(k)))
					keys, err := partitioning.Find(k_big)
					if err != nil {
						t.Error(err)
					}

					found := false
					for key, _ := range keys {
						if key.Cmp(k_big) == 0 {
							found = true
						}
					}
					if found {
						t.Logf("Expected to not find %v in %v", k_big, keys)
						t.Fail()
						return
					}
				}
			}
		}
	}
}

// var partitioning10 Partitioning = NewLruPartitioning(32, 4, 10)
// var partitioning100 Partitioning = NewLruPartitioning(32, 4, 100)
// var partitioning1000 Partitioning = NewLruPartitioning(32, 4, 1000)
// var partitioning10000 Partitioning = NewLruPartitioning(32, 4, 10000)
// var partitioning100000 Partitioning = NewLruPartitioning(32, 4, 100000)
// 
// func init() {
// 	for i := 0; i < 10; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 		partitioning10.Insert(key_big)
// 	}
// 	for i := 0; i < 100; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 		partitioning100.Insert(key_big)
// 	}
// 	for i := 0; i < 1000; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 		partitioning1000.Insert(key_big)
// 	}
// 	for i := 0; i < 10000; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 		partitioning10000.Insert(key_big)
// 	}
// 	for i := 0; i < 100000; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 		partitioning100000.Insert(key_big)
// 	}
// }
// 
// func Benchmark10(b *testing.B) {
// 	for i := 0; i < b.N; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 
// 		partitioning10.Insert(key_big)
// 		partitioning10.Find(key_big)
// 		partitioning10.Remove(key_big)
// 	}
// }
// 
// func Benchmark100(b *testing.B) {
// 	for i := 0; i < b.N; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 
// 		partitioning100.Insert(key_big)
// 		partitioning100.Find(key_big)
// 		partitioning100.Remove(key_big)
// 	}
// }
// 
// func Benchmark1000(b *testing.B) {
// 	for i := 0; i < b.N; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 
// 		partitioning1000.Insert(key_big)
// 		partitioning1000.Find(key_big)
// 		partitioning1000.Remove(key_big)
// 	}
// }
// 
// func Benchmark10000(b *testing.B) {
// 	for i := 0; i < b.N; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 
// 		partitioning10000.Insert(key_big)
// 		partitioning10000.Find(key_big)
// 		partitioning10000.Remove(key_big)
// 	}
// }
// 
// func Benchmark100000(b *testing.B) {
// 	for i := 0; i < b.N; i++ {
// 		key := rand.Uint32()
// 		key_big := NewKey(big.NewInt(int64(key)))
// 
// 		partitioning100000.Insert(key_big)
// 		partitioning100000.Find(key_big)
// 		partitioning100000.Remove(key_big)
// 	}
// }
