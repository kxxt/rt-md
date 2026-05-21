// MIT License
// Copyright (c) 2022 Jakub Pastuszek

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use chrono::offset::Local;
use chrono::{DateTime, FixedOffset};
use itertools::Itertools;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::{Error as IoError, Write};
use std::str::from_utf8;

#[derive(Debug)]
pub struct SyslogPriorityNameError;

impl Display for SyslogPriorityNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid syslog facility or severity name")
    }
}
impl Error for SyslogPriorityNameError {}

const FACILITY: [&str; 24] = [
    "kern", "user", "mail", "daemon", "auth", "syslog", "lpr", "news", "uucp", "cron", "authpriv",
    "ftp", "ntp", "audit", "alert", "clockd", "local0", "local1", "local2", "local3", "local4",
    "local5", "local6", "local7",
];

pub fn facility_by_id(facility: u8) -> Option<&'static str> {
    FACILITY.get(facility as usize).copied()
}

pub fn facility_by_name(facility: &str) -> Result<u8, SyslogPriorityNameError> {
    FACILITY
        .iter()
        .position(|name| *name == facility)
        .map(|pos| pos as u8)
        .ok_or(SyslogPriorityNameError)
}

const SEVERITY: [&str; 8] = [
    "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
];

pub fn severity_by_id(severity: u8) -> Option<&'static str> {
    SEVERITY.get(severity as usize).copied()
}

pub fn severity_by_name(facility: &str) -> Result<u8, SyslogPriorityNameError> {
    SEVERITY
        .iter()
        .position(|name| *name == facility)
        .map(|pos| pos as u8)
        .ok_or(SyslogPriorityNameError)
}

#[derive(Debug)]
pub struct SyslogHeader<'s> {
    pub facility: u8,
    pub severity: u8,
    pub timestamp: DateTime<FixedOffset>,
    pub hostname: Option<&'s [u8]>,
    pub tag: Option<&'s [u8]>,
}

// escape new lines to pereserve message boundries
pub fn write_syslog_message(
    out: &mut impl Write,
    message: &[u8],
    escape: &[u8],
) -> Result<(), IoError> {
    for (no, line) in message.split(|c| *c == b'\n').enumerate() {
        if no != 0 {
            out.write_all(escape)?;
        }
        out.write_all(line)?;
    }
    Ok(())
}

impl<'s> SyslogHeader<'s> {
    pub fn new(
        timestamp: DateTime<FixedOffset>,
        facility: &str,
        severity: &str,
        tag: &'s str,
    ) -> Result<SyslogHeader<'s>, SyslogPriorityNameError> {
        Ok(SyslogHeader {
            facility: facility_by_name(facility)?,
            severity: severity_by_name(severity)?,
            timestamp,
            hostname: None,
            tag: Some(tag.as_bytes()),
        })
    }

    pub fn new_raw(
        timestamp: DateTime<FixedOffset>,
        facility: u8,
        severity: u8,
        tag: &'s [u8],
    ) -> SyslogHeader<'s> {
        SyslogHeader {
            facility,
            severity,
            timestamp,
            hostname: None,
            tag: Some(tag),
        }
    }

    pub fn hostname(&mut self, hostname: &'s [u8]) {
        self.hostname = Some(hostname);
    }

    pub fn write(&self, out: &mut impl Write, timestamp_rfc3339: bool) -> Result<(), IoError> {
        write!(
            out,
            "<{}>{} ",
            ((self.facility as i32) << 3) + (self.severity as i32),
            if timestamp_rfc3339 {
                self.timestamp.format("%Y-%m-%dT%H:%M:%S%.6f%:z")
            } else {
                self.timestamp.format("%b %e %T")
            }
        )?;
        if let Some(hostname) = self.hostname {
            write_syslog_message(out, hostname, b"")?;
            out.write_all(b" ")?;
        }
        write_syslog_message(out, self.tag.unwrap_or(b"syslog"), b"")?;
        out.write_all(b": ")?;

        Ok(())
    }

    pub fn parse(
        message: &[u8],
        timestamp: Option<DateTime<FixedOffset>>,
    ) -> Option<(SyslogHeader<'_>, &[u8])> {
        let (pri, message) = message
            .strip_prefix(b"<")?
            .splitn(2, |c| *c == b'>')
            .collect_tuple()?;

        let pri: u8 = from_utf8(pri).ok()?.parse().ok()?;
        let facility = pri >> 3;
        let severity = pri & 0x7;

        // Some if hight precision timestamp was available
        let mut timestamp_rfc3339 = None;

        let message = if let (Some(b' '), Some(b' '), Some(b':'), Some(b':'), Some(b' ')) = (
            message.get(3),
            message.get(6),
            message.get(9),
            message.get(12),
            message.get(15),
        ) {
            // Apr 14 15:31:38
            message.split_at(16).1
        } else if let (Some(b'-'), Some(b'-'), Some(b'T'), Some((date, message))) = (
            message.get(4),
            message.get(7),
            message.get(10),
            message.splitn(2, |c| *c == b' ').collect_tuple(),
        ) {
            // 2022-04-14T15:34:45.327514+00:00
            let date = String::from_utf8_lossy(date);
            timestamp_rfc3339 = DateTime::parse_from_rfc3339(&date).ok();
            message
        } else if let Some((_sec_since_boot, message)) = message
            .strip_prefix(b"[")
            .and_then(|m| m.splitn(2, |c| *c == b']').collect_tuple())
        {
            // [23540.809085]
            message.strip_prefix(b" ").unwrap_or(message)
        } else {
            message
        };

        let (tag, message) =
            if let Some(tag_end) = message.windows(2).position(|c| *c == [b':', b' ']) {
                let (tag, message) = message.split_at(tag_end);
                (Some(tag), &message[2..])
            } else {
                (None, message)
            };

        Some((
            SyslogHeader {
                facility,
                severity,
                timestamp: timestamp_rfc3339
                    .or(timestamp)
                    .unwrap_or_else(|| Local::now().into()),
                hostname: None,
                tag,
            },
            message,
        ))
    }
}

#[derive(Debug)]
pub struct SyslogDatagram<'b> {
    buf_len: usize,
    datagram: &'b mut [u8],
}

impl<'b> Write for SyslogDatagram<'b> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        self.datagram.write(buf)
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.datagram.flush()
    }
}

impl<'b> SyslogDatagram<'b> {
    pub fn bytes_written(self) -> usize {
        self.buf_len - self.datagram.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal() {
        let ts = DateTime::parse_from_rfc3339("2022-04-14T15:34:45.327514+00:00").unwrap();
        let (hdr, msg) = SyslogHeader::parse(b"<5>hello world!", Some(ts)).unwrap();
        assert_eq!(facility_by_id(hdr.facility).unwrap(), "kern");
        assert_eq!(severity_by_id(hdr.severity).unwrap(), "notice");
        assert_eq!(hdr.timestamp, ts);
        assert_eq!(String::from_utf8(msg.to_vec()).unwrap(), "hello world!");
    }

    #[test]
    fn test_parse_klog() {
        let ts = DateTime::parse_from_rfc3339("2022-04-14T15:34:45.327514+00:00").unwrap();
        let (hdr, msg) = SyslogHeader::parse(
            b"<3>[23540.809085] usb 1-1: 3:1: cannot get min/max values for control 2 (id 3)",
            Some(ts),
        )
        .unwrap();
        assert_eq!(facility_by_id(hdr.facility).unwrap(), "kern");
        assert_eq!(severity_by_id(hdr.severity).unwrap(), "err");
        assert_eq!(hdr.timestamp, ts);
        assert_eq!(
            String::from_utf8(hdr.tag.unwrap().to_vec()).unwrap(),
            "usb 1-1"
        );
        assert_eq!(
            String::from_utf8(msg.to_vec()).unwrap(),
            "3:1: cannot get min/max values for control 2 (id 3)"
        );
    }

    #[test]
    fn test_parse_bsd() {
        let ts = DateTime::parse_from_rfc3339("2022-04-14T15:34:45.327514+00:00").unwrap();
        let (hdr, msg) =
            SyslogHeader::parse(b"<13>Apr 25 15:46:05 fred: hello world", Some(ts)).unwrap();
        assert_eq!(facility_by_id(hdr.facility).unwrap(), "user");
        assert_eq!(severity_by_id(hdr.severity).unwrap(), "notice");
        assert_eq!(hdr.timestamp, ts);
        assert_eq!(
            String::from_utf8(hdr.tag.unwrap().to_vec()).unwrap(),
            "fred"
        );
        assert_eq!(String::from_utf8(msg.to_vec()).unwrap(), "hello world");
    }

    #[test]
    fn test_parse_rfs3339() {
        let (hdr, msg) = SyslogHeader::parse(
            b"<13>2022-04-14T15:34:45.327514+00:00 fred: hello world",
            None,
        )
        .unwrap();
        assert_eq!(facility_by_id(hdr.facility).unwrap(), "user");
        assert_eq!(severity_by_id(hdr.severity).unwrap(), "notice");
        assert_eq!(
            hdr.timestamp,
            DateTime::parse_from_rfc3339("2022-04-14T15:34:45.327514+00:00").unwrap()
        );
        assert_eq!(
            String::from_utf8(hdr.tag.unwrap().to_vec()).unwrap(),
            "fred"
        );
        assert_eq!(String::from_utf8(msg.to_vec()).unwrap(), "hello world");
    }
}
