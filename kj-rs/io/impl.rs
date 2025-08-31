use crate::{AsyncInputStream, AsyncOutputStream, Result};

/// Performs a pump using `read()` and `write()`, without calling the stream's `pump_to()` nor
/// `try_pump_from()` methods.
///
/// This is intended to be used as a fallback by implementations of `pump_to()`
/// and `try_pump_from()` when they want to give up on optimization, but can't just call `pump_to()` again
/// because this would recursively retry the optimization. `unoptimized_pump_to()` should only be called
/// inside implementations of streams, never by the caller of a stream -- use the `pump_to()` method
/// instead.
///
/// `completed_so_far` is the number of bytes out of `amount` that have already been pumped. This is
/// provided for convenience for cases where the caller has already done some pumping before they
/// give up. Otherwise, a `.then()` would need to be used to add the bytes to the final result.
///
/// # Errors
///
/// Returns an error if reading from the input stream or writing to the output stream fails.
#[allow(clippy::cast_possible_truncation)]
pub async fn unoptimized_pump_to<I: AsyncInputStream, O: AsyncOutputStream>(
    input: &mut I,
    output: &mut O,
    amount: usize,
    completed_so_far: usize,
) -> Result<usize> {
    let mut buffer = [0u8; 4096];
    let mut total_pumped = completed_so_far;
    let mut remaining = amount.saturating_sub(completed_so_far);

    while remaining > 0 {
        let to_read = std::cmp::min(remaining, buffer.len());
        let bytes_read = input.try_read(&mut buffer[..to_read], 1).await?;

        if bytes_read == 0 {
            break; // EOF
        }

        output.write(&buffer[..bytes_read]).await?;
        total_pumped += bytes_read;
        remaining = remaining.saturating_sub(bytes_read);
    }

    Ok(total_pumped)
}
