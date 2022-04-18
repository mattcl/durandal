use anyhow::{bail, Result};
use nom::bytes::complete::tag;
use nom::character::complete::{char, u64};
use nom::sequence::delimited;
use nom::IResult;

pub fn new_task_parser(i: &str) -> Result<u64> {
    let out: IResult<&str, u64> = delimited(tag("Created task "), u64, char('.'))(i);
    match out {
        Ok((_, id)) => Ok(id),
        Err(e) => bail!("Could not parse id from created task: {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_created_task_id() {
        let input = "Created task 30.\n";
        let id = new_task_parser(input).unwrap();

        assert_eq!(id, 30);

        let input = "Created task 4.";
        let id = new_task_parser(input).unwrap();

        assert_eq!(id, 4);
    }
}
