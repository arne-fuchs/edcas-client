//TODO Make first character large of result
pub fn extract_word_from_standard(string: &String) -> String {
    string.split('_').last().unwrap_or(string).replace(';',"")
}