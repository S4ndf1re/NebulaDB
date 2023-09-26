pub(crate) fn two_randoms<T: PartialEq>(list: &Vec<T>) -> ((usize, &T), (usize, &T)) {
    let idx1 = rand::random::<usize>() % list.len();

    let mut idx2 = rand::random::<usize>() % list.len();

    while idx2 == idx1 && list[idx1] != list[idx2] {
        idx2 = rand::random::<usize>() % list.len();
    }

    ((idx1, &list[idx1]), (idx2, &list[idx2]))
}

/// assume two sorted lists
pub(crate) fn merge_by<'a, T>(a: Vec<(f64, &'a T)>, b: Vec<(f64, &'a T)>) -> Vec<(f64, &'a T)> {
    let mut idx_a = 0;
    let mut idx_b = 0;

    let mut result = vec![];

    while idx_a < a.len() && idx_b < b.len() {
        if a[idx_a].0 <= b[idx_b].0 {
            result.push(a[idx_a]);
            idx_a += 1;
        } else {
            result.push(b[idx_b]);
            idx_b += 1;
        }
    }

    while idx_a < a.len() {
        result.push(a[idx_a]);
        idx_a += 1;
    }

    while idx_b < b.len() {
        result.push(b[idx_b]);
        idx_b += 1;
    }

    result
}
