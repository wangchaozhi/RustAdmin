enum AudioOutputFormat {
  m4a('M4A', 'm4a'),
  mp3('MP3', 'mp3'),
  wav('WAV', 'wav');

  const AudioOutputFormat(this.label, this.extension);

  final String label;
  final String extension;
}

class VideoAudioConversionResult {
  const VideoAudioConversionResult({
    required this.outputPath,
    required this.fileName,
  });

  final String outputPath;
  final String fileName;
}
