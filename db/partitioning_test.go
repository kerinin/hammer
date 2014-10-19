package db

import (
	// "fmt"
	"testing"
	"reflect"
	"math/rand"
	"math/big"
)

func TestPartitioningPartitionEvenly(t *testing.T) {
	partitioning := NewPartitioning(32, 5)
	expected_partitions := make([]Partition, 4, 4)

	expected_partitions[0] = NewPartition(0, 8)
	expected_partitions[1] = NewPartition(8, 8)
	expected_partitions[2] = NewPartition(16, 8)
	expected_partitions[3] = NewPartition(24, 8)

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningPartitionUnevenly(t *testing.T) {
	partitioning := NewPartitioning(32, 7)
	expected_partitions := make([]Partition, 5, 5)

	expected_partitions[0] = NewPartition(0, 7)
	expected_partitions[1] = NewPartition(7, 7)
	expected_partitions[2] = NewPartition(14, 6)
	expected_partitions[3] = NewPartition(20, 6)
	expected_partitions[4] = NewPartition(26, 6)

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningPartitionTooMany(t *testing.T) {
	partitioning := NewPartitioning(4, 8)
	expected_partitions := make([]Partition, 3, 3)

	expected_partitions[0] = NewPartition(0, 2)
	expected_partitions[1] = NewPartition(2, 1)
	expected_partitions[2] = NewPartition(3, 1)

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningPartitionZero(t *testing.T) {
	partitioning := NewPartitioning(32, 0)
	expected_partitions := make([]Partition, 1, 1)

	expected_partitions[0] = NewPartition(0, 32)

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningPartitionWithNoBytes(t *testing.T) {
	partitioning := NewPartitioning(0, 0)
	expected_partitions := make([]Partition, 1, 1)

	expected_partitions[0] = NewPartition(0, 0)

	if !reflect.DeepEqual(expected_partitions, partitioning.partitions) {
		t.Logf("Expected partitions don't match actual (%v): %v", expected_partitions, partitioning.partitions)
		t.Fail()
	}
}

func TestPartitioningFindMissingKey(t *testing.T) {
	partitioning := NewPartitioning(8, 2)
	a := binary("11111111")
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
	partitioning := NewPartitioning(8, 2)
	a := binary("11111111")

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
	partitioning := NewPartitioning(4, 4)
	a := binary("00001111")
	b := binary("00001111")

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
	partitioning := NewPartitioning(4, 4)
	a := binary("00001111")
	b := binary("00001111")
	expected := map[*big.Int]uint{a: 0}

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
	partitioning := NewPartitioning(4, 4)
	a := binary("00001111")
	b := binary("00000111")
	expected := map[*big.Int]uint{a: 1}

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
	partitioning := NewPartitioning(8, 4)
	a := binary("00000000")
	b := binary("10000000")
	c := binary("10000001")
	d := binary("11000001")
	e := binary("11000011")

	expected := map[*big.Int]uint{b: 1, c: 2, d: 3, e: 4}

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
	partitioning := NewPartitioning(4, 2)
	a := binary("00001111")
	b := binary("00110011")

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
	partitioning := NewPartitioning(4, 4)
	a := binary("00001111")
	b := binary("00001111")
	c := binary("00001111")

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
	partitioning := NewPartitioning(4, 4)
	a := binary("00001111")

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
	partitioning := NewPartitioning(16, 4)

	var expected_present [65536]bool
	var expected_absent [65536]bool

	for i := 0; i < 100000; i ++ {
		j := rand.Uint32() % 16
		j_big := big.NewInt(int64(j))

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
		if i % 1000 == 0 {
			for k, b := range(expected_present) {
				if b {
					k_big := big.NewInt(int64(k))
					keys, err := partitioning.Find(k_big)
					if err != nil {
						t.Error(err)
					}

					found := false
					for key, _ := range(keys) {
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
			for k, b := range(expected_absent) {
				if b {
					k_big := big.NewInt(int64(k))
					keys, err := partitioning.Find(k_big)
					if err != nil {
						t.Error(err)
					}

					found := false
					for key, _ := range(keys) {
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
