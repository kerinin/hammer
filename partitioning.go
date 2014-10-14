package main

/*
import (
	"math"
	"math/big"

	"gopkg.in/fatih/set.v0"
)

type Partitioning struct {
	partitions []Partition
}

func NewPartitioning(bits uint, max_hamming_distance uint) Partitioning {
	var partition_count = math.Max(1, math.Min(bits, max_hamming_distance + 1))

	head_width := math.Ceil(bits / float64(partition_count))
	tail_width := math.Floor(bits / float64(partition_count))

	head_count := bits % partition_count
	tail_count := partition_count - head_count

	partitions := make([]Partition, 0, 0)

	for i := 0; i < head_count; i++ {
		shift := i * head_width
		mask := head_width

		partitions[i] = NewPartition(shift, mask)
	}

	for i := 0; i < tail_count; i++ {
		shift := (head_count * head_width) + (i * tail_width)
		mask := tail_width;

		partitions[head_count + i] = NewPartition(shift, mask)
	}

	return Partitioning{partitions: partitions}
}

func (p *Partitioning) Find(key big.Int) Set {
	results := set.New()

	for partition := range(p.partitions) {
		for key := partition.Find(key) {
			results.Add(key)
		}
	}

	return results
}

func (p *Partitioning) Insert(key big.Int) bool {
	any_inserted := false

	for partition := range(p.partitions) {
		any_inserted = any_inserted || partition.Insert(key)
	}

	return any_inserted
}

func (p *Partitioning) Remove(key big.Int) bool {
	any_removed := false

	for partition := range(p.partitions) {
		any_removed = any_removed || partition.Remove(key)
	}

	return any_inserted
}
*/
