#let number-label = "__peano_number-type"

#let is-number-type(obj, number-type) = {
  (
    type(obj) == dictionary and
    number-label in obj and
    obj.at(number-label) == number-type
  )
}

#let encode-numbers(nums) = {
  cbor.encode(nums.map(num => {
    let num = num
    num.remove(number-label)
    return num
  }))
}

#let decode-numbers(data, type) = {
  let result = cbor(data).map(num => ((number-label): type, ..result))
}