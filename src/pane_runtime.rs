//! Testable runtime for one pane's terminal session and parsed screen state.

use vt100::Parser;

use crate::terminal_session::{TerminalSession, TerminalSessionError, TerminalSize};

/// Coordinates terminal bytes, parser state, resize behavior, and redraw needs for one pane.
pub(crate) struct PaneRuntime<S> {
    session: S,
    parser: Parser,
}

impl<S> PaneRuntime<S>
where
    S: TerminalSession,
{
    /// Creates a pane runtime with parser dimensions matching the terminal session size.
    pub(crate) fn new(size: TerminalSize, session: S) -> Self {
        Self {
            session,
            parser: Parser::new(size.rows, size.cols, 0),
        }
    }

    /// Sends already-translated terminal input bytes into the pane's terminal session.
    pub(crate) fn write_input(&mut self, bytes: Vec<u8>) -> Result<(), TerminalSessionError> {
        self.session.write_input(bytes)
    }

    /// Drains all currently available terminal session output into the parser.
    ///
    /// Returns `true` when output changed the parsed screen and the pane should be redrawn.
    pub(crate) fn drain_output(&mut self) -> Result<bool, TerminalSessionError> {
        let mut changed = false;

        while let Some(bytes) = self.session.try_read_output()? {
            self.parser.process(&bytes);
            changed = true;
        }

        Ok(changed)
    }

    /// Resizes both the terminal session and the parser grid.
    pub(crate) fn resize(&mut self, size: TerminalSize) -> Result<(), TerminalSessionError> {
        self.session.resize(size)?;
        self.parser.set_size(size.rows, size.cols);
        Ok(())
    }

    /// Returns the parsed terminal screen state used by the renderer.
    pub(crate) fn parser(&self) -> &Parser {
        &self.parser
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

    use super::*;

    #[derive(Default)]
    struct FakeState {
        input: Vec<Vec<u8>>,
        output: VecDeque<Result<Vec<u8>, TerminalSessionError>>,
        resizes: Vec<TerminalSize>,
    }

    #[derive(Clone, Default)]
    struct FakeTerminalSession {
        state: Rc<RefCell<FakeState>>,
    }

    impl FakeTerminalSession {
        fn push_output(&self, bytes: &[u8]) {
            self.state.borrow_mut().output.push_back(Ok(bytes.to_vec()));
        }

        fn push_output_error(&self, error: TerminalSessionError) {
            self.state.borrow_mut().output.push_back(Err(error));
        }

        fn input(&self) -> Vec<Vec<u8>> {
            self.state.borrow().input.clone()
        }

        fn resizes(&self) -> Vec<TerminalSize> {
            self.state.borrow().resizes.clone()
        }
    }

    impl TerminalSession for FakeTerminalSession {
        fn write_input(&mut self, bytes: Vec<u8>) -> Result<(), TerminalSessionError> {
            self.state.borrow_mut().input.push(bytes);
            Ok(())
        }

        fn try_read_output(&mut self) -> Result<Option<Vec<u8>>, TerminalSessionError> {
            self.state.borrow_mut().output.pop_front().transpose()
        }

        fn resize(&mut self, size: TerminalSize) -> Result<(), TerminalSessionError> {
            self.state.borrow_mut().resizes.push(size);
            Ok(())
        }
    }

    fn terminal_size(rows: u16, cols: u16) -> TerminalSize {
        TerminalSize {
            rows,
            cols,
            pixel_width: 800,
            pixel_height: 600,
        }
    }

    #[test]
    fn write_input_forwards_bytes_to_terminal_session() {
        let session = FakeTerminalSession::default();
        let mut runtime = PaneRuntime::new(terminal_size(2, 8), session.clone());

        runtime.write_input(b"ls\r".to_vec()).unwrap();

        assert_eq!(session.input(), vec![b"ls\r".to_vec()]);
    }

    #[test]
    fn drain_output_processes_pending_bytes_and_reports_redraw() {
        let session = FakeTerminalSession::default();
        session.push_output(b"hi");
        let mut runtime = PaneRuntime::new(terminal_size(2, 8), session);

        assert!(runtime.drain_output().unwrap());
        assert_eq!(
            runtime.parser().screen().cell(0, 0).unwrap().contents(),
            "h"
        );
        assert_eq!(
            runtime.parser().screen().cell(0, 1).unwrap().contents(),
            "i"
        );
        assert!(!runtime.drain_output().unwrap());
    }

    #[test]
    fn drain_output_returns_terminal_session_errors() {
        let session = FakeTerminalSession::default();
        session.push_output_error(TerminalSessionError::OutputClosed);
        let mut runtime = PaneRuntime::new(terminal_size(2, 8), session);

        assert_eq!(
            runtime.drain_output().unwrap_err(),
            TerminalSessionError::OutputClosed
        );
    }

    #[test]
    fn resize_updates_terminal_session_and_parser_grid() {
        let session = FakeTerminalSession::default();
        let mut runtime = PaneRuntime::new(terminal_size(2, 8), session.clone());
        let next_size = terminal_size(4, 12);

        runtime.resize(next_size).unwrap();

        assert_eq!(session.resizes(), vec![next_size]);
        assert_eq!(runtime.parser().screen().size(), (4, 12));
    }
}
