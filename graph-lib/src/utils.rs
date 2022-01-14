pub(crate) fn remove_from_vec<T, VC: FnMut(&T) -> bool>(data: &mut Vec<T>, value_comparator: VC) {
    if let Some(position) = data.iter().position(value_comparator) {
        data.remove(position);
    }
}