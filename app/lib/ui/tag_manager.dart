import 'package:app/application_state.dart';
import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class TagManager extends StatefulWidget {
  @override
  _TagManagerState createState() => _TagManagerState();
}

class _TagManagerState extends State<TagManager> {
  List<Tag> tags = [];
  Future<void> _loadTags(BuildContext context) async {
    try {
      // Using the context to get the gRPC client and fetch the tags
      var response = await context
          .watch<ApplicationState>()
          .moneyViewClient
          .getTags(Empty());
      setState(() {
        tags = response.tags;
      });
    } catch (e) {
      print('Error fetching tags: $e');
    }
  }

  void _editTag(BuildContext context, Tag tag) {
    showDialog(
      context: context,
      builder: (context) {
        return TagEditDialog(
          tag: tag,
          onSave: (updatedTag) async {
            await context
                .read<ApplicationState>()
                .moneyViewClient
                .setTag(updatedTag);
            _loadTags(context);
          },
        );
      },
    );
  }

  void _createTag(BuildContext context) {
    _editTag(context, Tag(id: '', name: '', keyWords: []));
  }

  @override
  Widget build(BuildContext context) {
    _loadTags(context);
    return Center(
      child: Scaffold(
        appBar: AppBar(
          title: Text('Tag Manager'),
        ),
        body: ListView.builder(
          itemCount: tags.length,
          itemBuilder: (context, index) {
            final tag = tags[index];
            return ListTile(
              title: Text(tag.name),
              subtitle: Text('Keywords: ${tag.keyWords.join(", ")}'),
              trailing: IconButton(
                icon: Icon(Icons.edit),
                onPressed: () => _editTag(context, tag),
              ),
            );
          },
        ),
        floatingActionButton: FloatingActionButton(
          onPressed: () {
            _createTag(context);
          },
          child: Icon(Icons.add),
        ),
      ),
    );
  }
}

class TagEditDialog extends StatefulWidget {
  final Tag tag;
  final Function(Tag) onSave;

  TagEditDialog({required this.tag, required this.onSave});

  @override
  _TagEditDialogState createState() => _TagEditDialogState();
}

class _TagEditDialogState extends State<TagEditDialog> {
  late TextEditingController _nameController;
  late TextEditingController _keywordsController;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(text: widget.tag.name);
    _keywordsController =
        TextEditingController(text: widget.tag.keyWords.join(", "));
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(widget.tag.id.isEmpty ? 'Create Tag' : 'Edit Tag'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            controller: _nameController,
            decoration: InputDecoration(labelText: 'Name'),
          ),
          TextField(
            controller: _keywordsController,
            decoration:
                InputDecoration(labelText: 'Keywords (comma-separated)'),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text('Cancel'),
        ),
        TextButton(
          onPressed: () {
            final updatedTag = Tag(
              id: widget.tag.id.isEmpty ? widget.tag.name : widget.tag.id,
              name: _nameController.text,
              keyWords: _keywordsController.text
                  .split(',')
                  .map((e) => e.trim())
                  .toList(),
            );
            widget.onSave(updatedTag);
            Navigator.of(context).pop();
          },
          child: Text('Save'),
        ),
      ],
    );
  }
}
