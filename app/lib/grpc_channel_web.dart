import 'package:grpc/grpc_web.dart';
import 'package:grpc/service_api.dart';

ClientChannel getChannel() {
  return GrpcWebClientChannel.xhr(Uri.parse('http://localhost:50051'));
}
