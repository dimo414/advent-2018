use std::fs;
use std::collections::HashMap;

pub fn advent() {
    let data = read_data();
    println!("Metadata Sum: {}", metadata_sum(&data));
    println!("Root Value: {}", metadata_value(&data));
}

fn read_data() -> Vec<usize> {
    fs::read_to_string("data/day8.txt").expect("Cannot open")
        .trim().split(" ").map(|n| n.parse().unwrap()).collect()
}

fn metadata_sum(data: &[usize]) -> usize {
    let (end, sum) = metadata_sum_impl(data);
    assert_eq!(end, data.len());
    sum
}

fn metadata_sum_impl(data: &[usize]) -> (usize, usize) {
    let num_children = data[0];
    let num_metadata = data[1];
    let mut next_index = 2;
    let mut child_sum = 0;
    for _ in {0..num_children} {
        let (next_node, s) = metadata_sum_impl(&data[next_index..]);
        next_index += next_node;
        child_sum += s;
    }
    let next_node = next_index + num_metadata;
    let metadata_sum: usize = data[next_index..next_node].iter().sum();
    (next_node, child_sum + metadata_sum)
}

fn metadata_value(data: &[usize]) -> usize {
    let (end, sum) = metadata_value_impl(data);
    assert_eq!(end, data.len());
    sum
}

fn metadata_value_impl(data: &[usize]) -> (usize, usize) {
    let num_children = data[0];
    let num_metadata = data[1];
    let mut next_index = 2;
    if num_children == 0 {
        let next_node = next_index + num_metadata;
        let metadata_sum: usize = data[next_index..next_node].iter().sum();
        return (next_node, metadata_sum);
    }

    let mut child_values = HashMap::new();
    for i in {0..num_children} {
        let (next_node, v) = metadata_value_impl(&data[next_index..]);
        next_index += next_node;
        child_values.insert(i+1, v);
    }
    let mut value_sum: usize = 0;
    for i in {0..num_metadata} {
        value_sum += child_values.get(&data[next_index + i]).unwrap_or(&0);
    }
    let next_node = next_index + num_metadata;
    (next_node, value_sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file() {
        assert!(read_data().len() > 0);
    }

    #[test]
    fn sums() {
        assert_eq!(metadata_sum(&[0, 3, 10, 11, 12]), 33); // B
        assert_eq!(metadata_sum(&[0, 1, 99]), 99); // D
        assert_eq!(metadata_sum(&[1, 1, 0, 1, 99, 2]), 101); // C
        assert_eq!(metadata_sum(&[2, 3, 0, 3, 10, 11, 12, 1, 1, 0, 1, 99, 2, 1, 1, 2]), 138);
    }

    #[test]
    fn values() {
        assert_eq!(metadata_value(&[0, 3, 10, 11, 12]), 33); // B
        assert_eq!(metadata_value(&[0, 1, 99]), 99); // D
        assert_eq!(metadata_value(&[1, 1, 0, 1, 99, 2]), 2); // C
        assert_eq!(metadata_value(&[2, 3, 0, 3, 10, 11, 12, 1, 1, 0, 1, 99, 2, 1, 1, 2]), 66);
    }
}