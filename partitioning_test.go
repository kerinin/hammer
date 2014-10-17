package main

import (
	"testing"
	"reflect"

	"gopkg.in/fatih/set.v0"
)

func TestPartitioningPartitionEvenly(t *testing.T) {
	partitioning := NewPartitioning(32, 4)
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
	partitioning := NewPartitioning(32, 5)
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
	expected_partitions := make([]Partition, 4, 4)

	expected_partitions[0] = NewPartition(0, 1)
	expected_partitions[1] = NewPartition(1, 1)
	expected_partitions[2] = NewPartition(2, 1)
	expected_partitions[3] = NewPartition(3, 1)

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

	if keys.Size() != 0 {
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

	inserted, err := partitioning.Insert(a)
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}

	inserted, err = partitioning.Insert(a)
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
	expected := set.New(&a)

	_, err := partitioning.Insert(a)
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}

	keys, err := partitioning.Find(a)
	if keys.IsEqual(expected) {
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
	expected := set.New(&a)

	partitioning.Insert(a)
	keys, err := partitioning.Find(b)
	if keys.IsEqual(expected) {
		t.Logf("Find returned unexpected set (expected %v): %v", expected, keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestPartitioningFindPermutationsOfMultipleSimilarKeys(t *testing.T) {
}

func TestPartitioningDontFindPermutationOfInsertedKey(t *testing.T) {
	partitioning := NewPartitioning(4, 2)
	a := binary("00001111")
	b := binary("00110011")

	partitioning.Insert(a)
	keys, err := partitioning.Find(b)
	if keys.Size() != 0 {
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

	partitioning.Insert(a)

	removed, err := partitioning.Remove(a)
	if !removed {
		t.Logf("Remove returned false")
		t.Fail()
	}
	if err != nil {
		t.Logf("Remove returned error: %v", err)
		t.Fail()
	}

	keys, err := partitioning.Find(a)
	if keys.Size() != 0 {
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
}
