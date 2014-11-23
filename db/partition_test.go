package db

import (
	// "fmt"
	"reflect"
	"testing"
)

func TestPartitionFindMissingKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := NewKeyFromBinaryString("00001111")
	keys, err := partition.Find(a)

	if len(keys) != 0 {
		t.Logf("Find returned non-empty set: %v", keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestPartitionFirstInsertion(t *testing.T) {
	partition := NewPartition(4, 4)
	a := NewKeyFromBinaryString("00001111")

	inserted, err := partition.Insert(a)
	if !inserted {
		t.Logf("Insert returned false")
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestPartitionSecondInsertion(t *testing.T) {
	partition := NewPartition(4, 4)
	a := NewKeyFromBinaryString("00001111")
	b := NewKeyFromBinaryString("00001111")

	inserted, err := partition.Insert(a)
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}

	inserted, err = partition.Insert(b)
	if inserted {
		t.Logf("Insert returned true")
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestPartitionFindInsertedKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := NewKeyFromBinaryString("00001111")
	b := NewKeyFromBinaryString("00001111")
	expected := make(map[Key]uint)
	expected[a] = 0

	_, err := partition.Insert(a)
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}

	keys, err := partition.Find(b)
	if !reflect.DeepEqual(keys, expected) {
		t.Logf("Find returned unexpected set (expected %v): %v", expected, keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestPartitionFindPermutationOfInsertedKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := NewKeyFromBinaryString("11110000")
	b := NewKeyFromBinaryString("11100000")
	expected := make(map[Key]uint)
	expected[a] = 0

	partition.Insert(a)
	keys, err := partition.Find(b)
	if !reflect.DeepEqual(keys, expected) {
		t.Logf("Find returned unexpected set (expected %v): %v", expected, keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestPartitionRemoveInsertedKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := NewKeyFromBinaryString("00001111")
	b := NewKeyFromBinaryString("00001111")
	c := NewKeyFromBinaryString("00001111")

	partition.Insert(a)

	removed, err := partition.Remove(b)
	if !removed {
		t.Logf("Remove returned false")
		t.Fail()
	}
	if err != nil {
		t.Logf("Remove returned error: %v", err)
		t.Fail()
	}

	keys, err := partition.Find(c)
	if len(keys) != 0 {
		t.Logf("Find returned non-empty set: %v", keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestPartitionRemoveMissingKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := NewKeyFromBinaryString("00001111")

	removed, err := partition.Remove(a)
	if removed {
		t.Logf("Remove returned true")
		t.Fail()
	}
	if err != nil {
		t.Logf("Remove returned error: %v", err)
		t.Fail()
	}
}
