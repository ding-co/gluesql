use {
    crate::*,
    chrono::{NaiveDate, NaiveTime},
    gluesql_core::data::{IntervalError, ValueError},
    gluesql_core::prelude::Value::*,
};

test_case!(time, async move {
    run!(
        r#"
CREATE TABLE TimeLog (
    id INTEGER,
    time1 TIME,
    time2 TIME,
)"#
    );

    run!(
        r#"
INSERT INTO TimeLog VALUES
    (1, "12:30:00", "13:31:01.123"),
    (2, "9:2:1", "AM 08:02:01.001"),
    (3, "PM 2:59", "9:00:00 AM");
"#
    );

    let t = NaiveTime::from_hms_milli;
    let i = |h, m, s, ms| {
        gluesql_core::data::Interval::milliseconds(
            (t(h, m, s, ms) - NaiveTime::from_hms(0, 0, 0)).num_milliseconds(),
        )
    };

    test!(
        "SELECT id, time1, time2 FROM TimeLog;",
        Ok(select!(
            id  | time1           | time2
            I64 | Time            | Time;
            1     t(12, 30, 0, 0)   t(13, 31, 1, 123);
            2     t(9, 2, 1, 0)     t(8, 2, 1, 1);
            3     t(14, 59, 0, 0)   t(9, 0, 0, 0)
        ))
    );

    test!(
        "SELECT * FROM TimeLog WHERE time1 > time2",
        Ok(select!(
            id  | time1           | time2
            I64 | Time            | Time;
            2     t(9, 2, 1, 0)     t(8, 2, 1, 1);
            3     t(14, 59, 0, 0)   t(9, 0, 0, 0)
        ))
    );

    test!(
        "SELECT * FROM TimeLog WHERE time1 <= time2",
        Ok(select!(
            id  | time1           | time2
            I64 | Time            | Time;
            1     t(12, 30, 0, 0)   t(13, 31, 1, 123)
        ))
    );

    test!(
        r#"SELECT * FROM TimeLog WHERE time1 = TIME "14:59:00""#,
        Ok(select!(
            id  | time1           | time2
            I64 | Time            | Time;
            3     t(14, 59, 0, 0)   t(9, 0, 0, 0)
        ))
    );

    test!(
        r#"SELECT * FROM TimeLog WHERE time1 < "1:00 PM""#,
        Ok(select!(
            id  | time1           | time2
            I64 | Time            | Time;
            1     t(12, 30, 0, 0)   t(13, 31, 1, 123);
            2     t(9, 2, 1, 0)     t(8, 2, 1, 1)
        ))
    );

    test!(
        r#"SELECT * FROM TimeLog WHERE TIME "23:00:00.123" > "PM 1:00";"#,
        Ok(select!(
            id  | time1           | time2
            I64 | Time            | Time;
            1     t(12, 30, 0, 0)   t(13, 31, 1, 123);
            2     t(9, 2, 1, 0)     t(8, 2, 1, 1);
            3     t(14, 59, 0, 0)   t(9, 0, 0, 0)
        ))
    );

    test!(
        r#"SELECT
        id,
        time1 - time2 AS time_sub,
        time1 + INTERVAL "1" HOUR AS add,
        time2 - INTERVAL "250" MINUTE AS sub
        FROM TimeLog;"#,
        Ok(select!(
            id  | time_sub                      | add             | sub
            I64 | Interval                      | Time            | Time;
            1     i(1, 1, 1, 123).unary_minus()   t(13, 30, 0, 0)    t(9, 21, 1, 123);
            2     i(0, 59, 59, 999)               t(10, 2, 1, 0)     t(3, 52, 1, 1);
            3     i(5, 59, 0, 0)                  t(15, 59, 0, 0)    t(4, 50, 0, 0)
        ))
    );

    test!(
        r#"SELECT
            id,
            DATE "2021-01-05" + time2 AS timestamp
        FROM TimeLog LIMIT 1;"#,
        Ok(select!(
            id  | timestamp
            I64 | Timestamp;
            1     NaiveDate::from_ymd(2021, 1, 5).and_hms_milli(13, 31, 1, 123)
        ))
    );

    test!(
        r#"SELECT * FROM TimeLog WHERE time1 > time2 + INTERVAL "1" YEAR"#,
        Err(IntervalError::AddYearOrMonthToTime {
            time: t(13, 31, 1, 123).to_string(),
            interval: gluesql_core::data::Interval::years(1).into(),
        }
        .into())
    );

    test!(
        r#"SELECT * FROM TimeLog WHERE time1 > time2 - INTERVAL "1-2" YEAR TO MONTH"#,
        Err(IntervalError::SubtractYearOrMonthToTime {
            time: t(13, 31, 1, 123).to_string(),
            interval: gluesql_core::data::Interval::months(14).into(),
        }
        .into())
    );

    test!(
        r#"INSERT INTO TimeLog VALUES (1, "12345-678", "20:05:01")"#,
        Err(ValueError::FailedToParseTime("12345-678".to_owned()).into())
    );
});
