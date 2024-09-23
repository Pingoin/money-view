import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';

class BalanceChart extends StatelessWidget {
  const BalanceChart({
    super.key,
    required this.data,
    required this.threshold,
  });

  final List<BalanceInformation> data;
  final double threshold;

  List<BalanceInformation> reduceSmallEntries(
      List<BalanceInformation> values, double threshold) {
    double otherSum = 0;
    int otherCount = 0;
    List<BalanceInformation> reducedList = [];

    for (var value in values) {
      if (value.balance.abs() < threshold) {
        otherSum += value.balance.round();
        otherCount++;
      } else {
        value.balance = value.balance.roundToDouble();
        reducedList.add(value);
      }
    }

    // Füge den „Andere“-Eintrag hinzu, wenn es kleine Werte gibt
    if (otherSum.abs() > 0) {
      reducedList.add(BalanceInformation(
          name: "Sonstige",
          balance: otherSum.roundToDouble(),
          transactionCount: otherCount));
    }

    return reducedList;
  }

  @override
  Widget build(BuildContext context) {
    // Erstelle eine Liste der Farben für jedes Segment
    final List<Color> sectionColors = List.generate(data.length,
        (index) => Colors.primaries[index % Colors.primaries.length]);
    List<BalanceInformation> small = reduceSmallEntries(data, threshold);

    return Column(children: [
      SizedBox(
        width: 200.0,
        height: 200.0,
        child: PieChart(PieChartData(
          centerSpaceRadius: 10,
          borderData: FlBorderData(show: false),
          sections: List.generate(small.length, (index) {
            final entry = small[index];
            return PieChartSectionData(
              color: Colors.primaries[
                  index % Colors.primaries.length], // Nutze eine Farbpalette
              value: entry.balance,
              title: '${entry.balance} €',
              radius: 100,
              titleStyle: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.bold,
                color: Colors.white,
              ),
            );
          }),
        )),
      ),
      Column(
        children: List.generate(small.length, (index) {
          final entry = small[index];
          return Row(
            mainAxisAlignment: MainAxisAlignment.start,
            children: [
              // Farbige Box für das Segment
              Container(
                width: 16,
                height: 16,
                color: sectionColors[index],
              ),
              SizedBox(width: 8),
              // Beschriftung
              Text(entry.name),
            ],
          );
        }),
      ),
    DataTable(
            columns: [
              DataColumn(label: Text('Name')),
              DataColumn(label: Text('Balance')),
              DataColumn(label: Text('Transaction Count')),
            ],
            rows: data.map((data) {
              return DataRow(cells: [
                DataCell(Text(data.name)),
                DataCell(Text(data.balance.toStringAsFixed(2))),
                DataCell(Text(data.transactionCount.toString())),
              ]);
            }).toList(),
          ),
    
    ]);
  }
}
