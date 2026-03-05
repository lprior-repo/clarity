import gleam/dict
import gleeunit
import gleeunit/should
import intent/answer_loader
import simplifile

pub fn main() {
  gleeunit.main()
}

pub fn load_from_file_flattens_nested_answers_test() {
  let path = "/tmp/intent-answer-loader-test.json"
  let content =
    "{\"r1-user-api-1\":\"Build authentication API\",\"security\":{\"auth_method\":\"JWT\"},\"limits\":{\"requests_per_minute\":60}}"

  let assert Ok(Nil) = simplifile.write(path, content)
  let assert Ok(answers) = answer_loader.load_from_file(path)

  dict.get(answers, "r1-user-api-1")
  |> should.equal(Ok("Build authentication API"))

  dict.get(answers, "security.auth_method")
  |> should.equal(Ok("JWT"))

  dict.get(answers, "auth_method")
  |> should.equal(Ok("JWT"))

  dict.get(answers, "limits.requests_per_minute")
  |> should.equal(Ok("60"))

  let _ = simplifile.delete(path)
  Nil
}

pub fn load_from_file_missing_path_returns_error_test() {
  case answer_loader.load_from_file("/tmp/does-not-exist-intent-answers.json") {
    Error(answer_loader.FileNotFound(_)) -> True |> should.equal(True)
    _ -> False |> should.equal(True)
  }
}
