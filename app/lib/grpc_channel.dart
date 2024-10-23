import 'package:grpc/grpc.dart';

ClientChannel getChannel() {
  return ClientChannel('192.168.178.38',
      port: 50051,
      options:
          const ChannelOptions(credentials: ChannelCredentials.insecure()));
}
