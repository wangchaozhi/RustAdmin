enum AudioOutputFormat {
  m4a('M4A', 'm4a', 'audio/mp4'),
  mp3('MP3', 'mp3', 'audio/mpeg'),
  wav('WAV', 'wav', 'audio/wav');

  const AudioOutputFormat(this.label, this.extension, this.contentType);

  final String label;
  final String extension;
  final String contentType;
}

class VideoAudioConversionResult {
  const VideoAudioConversionResult({
    required this.outputPath,
    required this.fileName,
  });

  final String outputPath;
  final String fileName;
}
