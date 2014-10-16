package main

import (
	"testing"
	"math/big"

	"gopkg.in/fatih/set.v0"
)

/*
 * Converts a string to a big.Int, like "0011" => 3
 */
func binary(s string) *big.Int {
	bigint := big.NewInt(0)

	for i, r := range(s) {
		if r == '1' {
			mask := big.NewInt(1)
			mask.Lsh(mask, uint(i))
			bigint.Or(bigint, mask)
		}
	}

	return bigint
}

func TestFindMissingKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := binary("00001111")
	keys, err := partition.Find(a)

	if keys.Size() != 0 {
		t.Logf("Find returned non-empty set: %v", keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestFirstInsertion(t *testing.T) {
	partition := NewPartition(4, 4)
	a := binary("00001111")

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

func TestSecondInsertion(t *testing.T) {
	partition := NewPartition(4, 4)
	a := binary("00001111")

	inserted, err := partition.Insert(a)
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}

	inserted, err = partition.Insert(a)
	if inserted {
		t.Logf("Insert returned true")
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestFindInsertedKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := binary("00001111")
	expected := set.New(&a)

	_, err := partition.Insert(a)
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}

	keys, err := partition.Find(a)
	if keys.IsEqual(expected) {
		t.Logf("Find returned unexpected set (expected %v): %v", expected, keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestFindPermutationOfInsertedKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := binary("00001111")
	b := binary("00000111")
	expected := set.New(&a)

	partition.Insert(a)
	keys, err := partition.Find(b)
	if keys.IsEqual(expected) {
		t.Logf("Find returned unexpected set (expected %v): %v", expected, keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Insert returned error: %v", err)
		t.Fail()
	}
}

func TestRemoveInsertedKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := binary("00001111")

	partition.Insert(a)

	removed, err := partition.Remove(a)
	if !removed {
		t.Logf("Remove returned false")
		t.Fail()
	}
	if err != nil {
		t.Logf("Remove returned error: %v", err)
		t.Fail()
	}

	keys, err := partition.Find(a)
	if keys.Size() != 0 {
		t.Logf("Find returned non-empty set: %v", keys)
		t.Fail()
	}
	if err != nil {
		t.Logf("Find returned error: %v", err)
		t.Fail()
	}
}

func TestRemoveMissingKey(t *testing.T) {
	partition := NewPartition(4, 4)
	a := binary("00001111")

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
