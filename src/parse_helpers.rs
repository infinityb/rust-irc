pub fn first_line(input: &[u8]) -> &[u8] {
    let mut end_idx = None;
    for (idx, &chr) in input.iter().enumerate() {
        if chr == b'\n' {
            end_idx = Some(idx);
            break;
        }
    }
    if let Some(idx) = end_idx {
        if idx > 0 && input[idx-1] == b'\r' {
            end_idx = Some(idx - 1);
        }
    }
    match end_idx {
        Some(idx) => &input[..idx],
        None => input,
    }
}

pub fn find_character(input: &[u8], byte: u8, offset: usize) -> Option<usize> {
    let mut end_idx = None;
    for (idx, &chr) in input.iter().enumerate().skip(offset) {
        if chr == byte {
            end_idx = Some(idx);
            break;
        }
    }
    end_idx
}

pub fn consume_whitespace(input: &[u8]) -> &[u8] {
    let mut output = input;
    for (idx, &chr) in input.iter().enumerate() {
        output = &input[idx..];
        if chr != b' ' {
            break;
        }
    }
    output
}

pub fn split_prefix(input: &[u8]) -> (&[u8], &[u8]) {
    if input[0] == b':' {
        let end_idx = find_character(input, b' ', 0);
        match end_idx {
            Some(idx) => (&input[..idx], consume_whitespace(&input[idx+1..])),
            None => (input, &[]),
        }
    } else {
        (&[], input)
    }
}

pub fn parse_prefix(input: &[u8]) -> Result<(&[u8], &[u8], &[u8]), ()> {
    if input.len() == 0 {
        return Err(());
    }
    if input[0] != b':' {
        return Err(());
    }

    let nick_start = 1;
    let nick_end = try!(find_character(input, b'!', nick_start).ok_or(()));

    let user_start = nick_end + 1;
    let user_end = try!(find_character(input, b'@', user_start).ok_or(()));;

    if is_valid_user(&input[user_start..user_end]) {
        return Err(());
    }

    let host_start = user_end + 1;

    Ok((
        &input[nick_start..nick_end],
        &input[user_start..user_end],
        &input[host_start..],
    ))
}

pub fn split_command(input: &[u8]) -> (&[u8], &[u8]) {
    let end_idx = find_character(input, b' ', 0);
    match end_idx {
        Some(idx) => (&input[..idx], consume_whitespace(&input[idx+1..])),
        None => (input, &[]),
    }
}

pub fn split_arg(input: &[u8]) -> (&[u8], &[u8]) {
    if input.len() == 0 {
        return (input, input);
    }
    if input[0] == b':' {
        (&input[1..], &[])
    } else {
        let end_idx = find_character(input, b' ', 0);
        match end_idx {
            Some(idx) => (&input[..idx], consume_whitespace(&input[idx+1..])),
            None => (input, &[]),
        }
    }
}

pub fn is_valid_nick(nick: &[u8]) -> bool {
    // FIXME: this is wrong.. see RFC.
    for &byte in nick.iter() {
        if !is_non_white(byte) {
            return false;
        }
    }
    return true;
}

pub fn is_non_white(byte: u8) -> bool {
    !(byte == 0x00 || byte == 0x0A || byte == 0x0D || byte == 0x20)
}

pub fn is_valid_user(user: &[u8]) -> bool {
    for &byte in user.iter() {
        if !is_non_white(byte) {
            return false;
        }
    }
    return true;
}

pub fn is_valid_prefix_byte(byte: u8) -> bool {
    is_non_white(byte)
}

pub fn is_valid_prefix(prefix: &[u8]) -> bool {
    for &byte in prefix.iter() {
        if !is_valid_prefix_byte(byte) {
            return false;
        }
    }
    return true;
}

pub fn is_valid_command(command: &[u8]) -> bool {
    for &byte in command.iter() {
        if 0x80 <= byte {
            return false;
        }
    }
    return true;
}
