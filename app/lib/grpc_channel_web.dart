import 'package:grpc/grpc_web.dart';
// ignore: avoid_web_libraries_in_flutter
import 'dart:html';
import 'package:grpc/service_api.dart';

ClientChannel getChannel() {
  String hostname = window.location.hostname ?? 'localhost';
  
  return GrpcWebClientChannel.xhr(Uri.parse('http://$hostname:50051'));
}
