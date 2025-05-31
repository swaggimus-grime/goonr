
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InputType {
    Points3D = 0,
    Images = 1,
    Cameras = 2,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InputFormat {
    Binary = 0,
    Text = 1,
}

#[derive(Debug)]
pub enum InputData {
    Images(HashMap<i32, Image>),
    Points3D(HashMap<i64, Point3D>),
    Cameras(HashMap<i32, Camera>),
}

struct InputFile {
    parser: Box<dyn Parseable>,
    input_format: InputFormat,
    path: PathBuf,
}

impl InputFile {
    async fn new(parser: Box<dyn Parseable>, input_format: InputFormat, path: PathBuf) -> io::Result<Self> {
        Ok(Self {
            parser,
            input_format,
            path
        })
    }

    pub async fn parse(&self) -> io::Result<InputData> {
        let file = tokio::fs::File::open(self.path.clone()).await?;
        let reader = tokio::io::BufReader::new(file);
        match &self.input_format {
            InputFormat::Binary => Ok(self.parser.parse_bin(reader).await?),
            InputFormat::Text => Ok(self.parser.parse_txt(reader).await?)
        }
    }
}

