import 'package:flutter/foundation.dart';
import 'package:grpc/grpc_web.dart';
// ignore: avoid_web_libraries_in_flutter
import 'package:grpc/service_api.dart';

ClientChannel getChannel() {  
String path = kDebugMode?"http://localhost:8080":"/";
  return GrpcWebClientChannel.xhr(Uri.parse(path));
}
