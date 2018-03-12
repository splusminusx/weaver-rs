use kafka::error::Error as KafkaError;
use std::io;
use std::num::ParseIntError;
use toml::de;

error_chain! {
    foreign_links {
        Io(io::Error) #[doc="Input/Output error"];
        Kafka(KafkaError) #[doc="An error as reported by Kafka client"];
        TomlDeserialize(de::Error) #[doc="An error as reported by Toml deserializer"];
        ParseNum(ParseIntError) #[doc="Error while parsing number from string."];
    }

    errors {
        OtherError(error_string: String) {
            description("Other Error")
            display("Other Error ({:?})", error_string)
        }
    }
}
