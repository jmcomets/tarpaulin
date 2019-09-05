use std::{io, fs, str::FromStr, path::Path};
use cargo::core::compiler::CompileMode;
use clap::arg_enum;
use coveralls_api::CiService;
use void::Void;

arg_enum! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
    pub enum RunType {
        Tests,
        Doctests,
    }
}

#[derive(Default, Debug)]
pub struct OutputFile(Option<String>);

trait AsPath {
    fn as_path(&self) -> &Path;
}

impl<P> AsPath for P where P: AsRef<Path> {
    fn as_path(&self) -> &Path {
        self.as_ref()
    }
}

impl OutputFile {
    pub fn create_or_else<P: AsRef<Path>>(&self, path: P) -> io::Result<fs::File> {
        let path = self.0.as_ref().map(|path| path.as_path()).unwrap_or(path.as_ref());
        fs::File::create(path)
    }

    pub fn create_or_stdout(&self) -> io::Result<Box<io::Write>> {
        Ok(if let Some(path) = self.0.as_ref() {
            Box::new(fs::File::create(path.as_path())?)
        } else {
            Box::new(io::stdout())
        })
    }
}

impl FromStr for OutputFile {
    /// This can never fail, so the error type is uninhabited.
    type Err = Void;

    #[inline]
    fn from_str(x: &str) -> Result<OutputFile, Self::Err> {
        Ok(OutputFile(Some(x.to_string())))
    }
}

arg_enum! {
    #[derive(Debug)]
    pub enum OutputType {
        Json,
        Toml,
        Xml,
        Html,
    }
}

pub struct Ci(pub CiService);

impl From<RunType> for CompileMode {
    fn from(run: RunType) -> Self {
        match run {
            RunType::Tests => CompileMode::Test,
            RunType::Doctests => CompileMode::Doctest,
        }
    }
}

impl FromStr for Ci {
    /// This can never fail, so the error type is uninhabited.
    type Err = Void;

    #[inline]
    fn from_str(x: &str) -> Result<Ci, Self::Err> {
        match x {
            "circle-ci" => Ok(Ci(CiService::Circle)),
            "codeship" => Ok(Ci(CiService::Codeship)),
            "jenkins" => Ok(Ci(CiService::Jenkins)),
            "semaphore" => Ok(Ci(CiService::Semaphore)),
            "travis-ci" => Ok(Ci(CiService::Travis)),
            "travis-pro" => Ok(Ci(CiService::TravisPro)),
            other => Ok(Ci(CiService::Other(other.to_string()))),
        }
    }
}
